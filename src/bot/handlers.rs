use crate::bot::processor::{ban_user, kick_user, mute_user, test_redis};

use std::str::FromStr;

use chrono::Duration;
use teloxide::{prelude::*, utils::command::BotCommands};

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
