use chrono::Duration;
use teloxide::{prelude::*, types::ChatPermissions};

use super::redis::fetch_an_integer;

// Test the Redis cache
pub async fn test_redis(bot: Bot, msg: Message) -> ResponseResult<()> {
    let result = fetch_an_integer();
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
