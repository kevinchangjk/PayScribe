use std::error::Error;
use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::Me,
    utils::command::BotCommands,
    RequestError,
};

use super::processor::ProcessError;

/* Handler is the front-facing agent of the bot.
 * It receives messages and commands from the user, and handles user interaction.
 * All user interaction, including sending and crafting of messages, is done here.
 * It communicates only with the Processor, which executes the commands.
 * User exceptions are handled in this module. Processor may propagate some errors here.dialogue
 */

/* Types */
pub type UserDialogue = Dialogue<State, InMemStorage<State>>;
pub type BotError = Box<dyn std::error::Error + Send + Sync>;
pub type HandlerResult = Result<(), BotError>;

/*
#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error("User error: {0}")]
    UserError(String),
    #[error("Process error: {0}")]
    ProcessError(ProcessError),
    #[error("Request error: {0}")]
    RequestError(RequestError),
}

impl From<RequestError> for BotError {
    fn from(request_error: RequestError) -> BotError {
        BotError::RequestError(request_error)
    }
}
*/

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveAge {
        full_name: String,
    },
    ReceiveLocation {
        full_name: String,
        age: u8,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Show this help message.")]
    Help,
    #[command(description = "Add a payment entry for the group.")]
    AddPayment,
}

/* Utility functions */

/* Main Dispatch function */
pub async fn run_dispatcher(bot: Bot) {
    Dispatcher::builder(
        bot,
        dptree::entry().branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(dptree::case![State::Start].endpoint(handle_start)),
        ),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

/* Endpoint handler functions */
pub async fn handle_start(bot: Bot, dialogue: UserDialogue, msg: Message, me: Me) -> HandlerResult {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(command) => {
                do_action(bot, msg, dialogue, command).await?;
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

/* Command Interaction functions */
pub async fn do_action(
    bot: Bot,
    msg: Message,
    dialogue: UserDialogue,
    cmd: Command,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => action_help(bot, msg).await?,
        Command::AddPayment => action_add_payment(bot, msg).await?,
    };

    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot will ask for user to send messages to fill in required information,
 * before presenting the compiled information for confirmation with a menu.
 */
pub async fn action_add_payment(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "Add payment!").await?;
    Ok(())
}
