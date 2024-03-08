use chrono::Duration;
use teloxide::{prelude::*, types::ChatPermissions};

use super::redis::{add_balance, add_chat, add_user, test_redis_connection};

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

// Adds a user to Redis cache
pub async fn add_user_redis(bot: Bot, msg: Message) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    let user_id = user.id.to_string();
    let username = user.username.as_ref().unwrap();
    let result = add_user(&user_id, &username);
    if result.is_ok() {
        bot.send_message(msg.chat.id, "User added to Redis").await?;
        log::info!("User added to Redis: {:?}", user);
    } else {
        bot.send_message(msg.chat.id, "Failed to add user to Redis")
            .await?;
        log::error!("Failed to add user to Redis: {:?}", result);
    }
    Ok(())
}

// Adds a chat to Redis cache
pub async fn add_chat_redis(bot: Bot, msg: Message) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    let user_id = user.id.to_string();
    let chat_id = msg.chat.id.to_string();
    let result = add_chat(&chat_id, &user_id);
    if result.is_ok() {
        bot.send_message(msg.chat.id, "Chat added to Redis").await?;
        log::info!("Chat added to Redis: {:?}", msg.chat);
    } else {
        bot.send_message(msg.chat.id, "Failed to add chat to Redis")
            .await?;
        log::error!("Failed to add chat to Redis: {:?}", result);
    }
    Ok(())
}

// Adds a balance to Redis cache
pub async fn add_balance_redis(bot: Bot, msg: Message) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    let user_id = user.id.to_string();
    let chat_id = msg.chat.id.to_string();
    let result = add_balance(&chat_id, &user_id);
    if result.is_ok() {
        bot.send_message(msg.chat.id, "Balance added to Redis")
            .await?;
        log::info!("Balance added to Redis: {:?}", msg.chat);
    } else {
        bot.send_message(msg.chat.id, "Failed to add balance to Redis")
            .await?;
        log::error!("Failed to add balance to Redis: {:?}", result);
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
