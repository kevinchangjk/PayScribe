use std::str::FromStr;

use chrono::Duration;
use teloxide::{prelude::*, types::ChatPermissions, utils::command::BotCommands};

use super::redis::test_redis_connection;

/* Handler is the front-facing agent of the bot.
 * It receives messages and commands from the user, and handles user interaction.
 * All user interaction, including sending and crafting of messages, is done here.
 * It communicates only with the Processor, which executes the commands.
 * User exceptions are handled in this module. Processor may propagate some errors here.
 */

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Use commands in format /%command% %num% %unit%",
    parse_with = "split"
)]
pub enum Command {
    #[command(description = "kick user from chat.")]
    Kick,
    #[command(description = "ban user in chat.")]
    Ban { time: u64, unit: UnitOfTime },
    #[command(description = "mute user in chat.")]
    Mute { time: u64, unit: UnitOfTime },
    #[command(description = "shows this message.")]
    Help,
    #[command(description = "test connection with redis cache.")]
    Redis,
}

#[derive(Clone)]
pub enum UnitOfTime {
    Seconds,
    Minutes,
    Hours,
}

impl FromStr for UnitOfTime {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "h" | "hours" => Ok(UnitOfTime::Hours),
            "m" | "minutes" => Ok(UnitOfTime::Minutes),
            "s" | "seconds" => Ok(UnitOfTime::Seconds),
            _ => Err("Allowed units: h, m, s"),
        }
    }
}

// Calculates time of user restriction.
fn calc_restrict_time(time: u64, unit: UnitOfTime) -> Duration {
    match unit {
        UnitOfTime::Hours => Duration::hours(time as i64),
        UnitOfTime::Minutes => Duration::minutes(time as i64),
        UnitOfTime::Seconds => Duration::seconds(time as i64),
    }
}

pub async fn do_action(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Kick => kick_user(bot, msg).await?,
        Command::Ban { time, unit } => ban_user(bot, msg, calc_restrict_time(time, unit)).await?,
        Command::Mute { time, unit } => mute_user(bot, msg, calc_restrict_time(time, unit)).await?,
        Command::Redis => test_redis(bot, msg).await?,
    };

    Ok(())
}

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
