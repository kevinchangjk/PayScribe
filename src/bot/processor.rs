use chrono::Duration;
use teloxide::{prelude::*, types::ChatPermissions};

use super::{
    optimizer::optimize_debts,
    redis::{
        add_payment_entry, test_redis_connection, update_chat, update_chat_balances,
        update_chat_debts, update_user, CrudError, Debt, Payment, UserBalance,
    },
};

/* Processor is the overall logic center of the bot.
 * It handles the main logic, communicating with the front-facing handler
 * and the back-facing redis manager.
 * It defines and executes the main functions required of the bot,
 * and handles exceptions and errors in the back.
 */

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ProcessError {
    #[error("Database CRUD error: {0}")]
    CrudError(CrudError),
}

// Implement the From trait to convert from CrudError to ProcessError
impl From<CrudError> for ProcessError {
    fn from(crud_error: CrudError) -> ProcessError {
        ProcessError::CrudError(crud_error)
    }
}

/* Add a new payment entry in a group chat.
 * Execution flow: Updates relevant users, updates chat.
 * Adds payment entry, updates balances, updates group debts.
 */
pub async fn add_payment(
    msg: Message,
    description: &str,
    creditor: &str,
    total: f64,
    debts: Vec<(String, f64)>,
) -> Result<(), ProcessError> {
    let chat_id = msg.chat.id.to_string();
    let mut all_users = vec![creditor.to_string()];

    for (user, _) in debts.iter() {
        all_users.push(user.to_string());
    }

    // Update all users included in payment
    for user in all_users.iter() {
        update_user(user, &chat_id, None)?;
    }

    // Add message sender to the list of users
    if let Some(user) = msg.from() {
        if let Some(username) = &user.username {
            update_user(username, &chat_id, Some(&user.id.to_string()))?;
            all_users.push(username.to_string());
        }
    }

    // Update chat
    update_chat(&chat_id, all_users)?;

    // Add payment entry
    let payment = Payment {
        description: description.to_string(),
        datetime: msg.date.to_string(),
        creditor: creditor.to_string(),
        total,
        debts: debts.clone(),
    };
    add_payment_entry(&chat_id, &payment)?;

    // Update balances
    let changes = debts
        .iter()
        .map(|(user, amount)| UserBalance {
            username: user.to_string(),
            balance: *amount,
        })
        .collect();
    let balances = update_chat_balances(&chat_id, changes)?;

    // Update group debts
    let debts = optimize_debts(balances);
    update_chat_debts(&chat_id, debts)?;

    Ok(())
}

/* TODO: Old functions, remove later */

// Test the Redis cache
pub async fn test_redis(bot: Bot, msg: Message) -> ResponseResult<()> {
    let result = test_redis_connection();
    if result.is_ok() {
        bot.send_message(msg.chat.id, "Redis connected").await?;
    } else {
        bot.send_message(msg.chat.id, "Redis disconnected").await?;
    }
    Ok(())
}

// Kick a user with a replied message.
pub async fn kick_user(bot: Bot, msg: Message) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            // bot.unban_chat_member can also kicks a user from a group chat.
            bot.unban_chat_member(msg.chat.id, replied.from().unwrap().id)
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Use this command in reply to another message")
                .await?;
        }
    }
    Ok(())
}

// Ban a user with replied message.
pub async fn ban_user(bot: Bot, msg: Message, time: Duration) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            bot.kick_chat_member(
                msg.chat.id,
                replied.from().expect("Must be MessageKind::Common").id,
            )
            .until_date(msg.date + time)
            .await?;
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "Use this command in a reply to another message!",
            )
            .await?;
        }
    }
    Ok(())
}

// Mute a user with a replied message.
pub async fn mute_user(bot: Bot, msg: Message, time: Duration) -> ResponseResult<()> {
    match msg.reply_to_message() {
        Some(replied) => {
            bot.restrict_chat_member(
                msg.chat.id,
                replied.from().expect("Must be MessageKind::Common").id,
                ChatPermissions::empty(),
            )
            .until_date(msg.date + time)
            .await?;
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "Use this command in a reply to another message!",
            )
            .await?;
        }
    }
    Ok(())
}
