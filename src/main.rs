mod bot;
use bot::do_action;
use bot::Command;
use teloxide::repls::CommandReplExt;

// Derive BotCommands to parse text with a command into this enumeration.
//
// 1. `rename_rule = "lowercase"` turns all the commands into lowercase letters.
// 2. `description = "..."` specifies a text before all the commands.
//
// That is, you can just call Command::descriptions() to get a description of
// your commands in this format:
// %GENERAL-DESCRIPTION%
// %PREFIX%%COMMAND% - %DESCRIPTION%

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting PayScribe bot...");

    let bot = teloxide::Bot::from_env();

    Command::repl(bot, do_action).await;
}
