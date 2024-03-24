use teloxide::{prelude::*, utils::command::BotCommands};

use super::super::dispatcher::{Command, HandlerResult, UserDialogue};

pub const UNKNOWN_ERROR_MESSAGE: &str = "An unknown error occurred. Please try again.";
pub const NO_TEXT_MESSAGE: &str = "Please reply in text.\n\n";
pub const DEBT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and the amounts as follows: \n\n@user1 amount1, @user2 amount2, etc.\n\n";
const INVALID_STATE_MESSAGE: &str = "Unable to handle the message. Type /help to see the usage.";
const START_MESSAGE: &str = "Hi, I'm PayScribe!";

/* Invalid state.
 */
pub async fn invalid_state(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, INVALID_STATE_MESSAGE).await?;
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, START_MESSAGE).await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

/* Cancel command.
 * Called when state is at start, thus nothing to cancel.
 */
pub async fn action_cancel(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Nothing to cancel!").await?;
    dialogue.exit().await?;
    Ok(())
}
