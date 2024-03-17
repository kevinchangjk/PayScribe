use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};

use super::processor::ProcessError;

/* Handler is the front-facing agent of the bot.
 * It receives messages and commands from the user, and handles user interaction.
 * All user interaction, including sending and crafting of messages, is done here.
 * It communicates only with the Processor, which executes the commands.
 * User exceptions are handled in this module. Processor may propagate some errors here.
 */

/* Types */
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum BotError {
    #[error("User error: {0}")]
    UserError(String),
    #[error("Process error: {0}")]
    ProcessError(ProcessError),
}

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

pub type UserDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), BotError>;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Add a payment entry for the group.")]
    AddPayment,
}

/* Utility functions */

/* Main handler function */
pub fn handler() -> UpdateHandler<BotError> {
    dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(handle_start)),
    )
}

/* Endpoint handler functions */
pub async fn handle_start(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    Ok(())
}

/* Command Interaction functions */
pub async fn do_action(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::AddPayment => action_add_payment(bot, msg).await?,
    };

    Ok(())
}

/* Add a payment entry in a group chat.
 * Bot will ask for user to send messages to fill in required information,
 * before presenting the compiled information for confirmation with a menu.
 */
pub async fn action_add_payment(bot: Bot, msg: Message) -> ResponseResult<()> {
    Ok(())
}
