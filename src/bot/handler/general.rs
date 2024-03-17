use teloxide::{prelude::*, utils::command::BotCommands};

use super::super::dispatcher::{Command, HandlerResult, UserDialogue};

const INVALID_STATE_MESSAGE: &str = "Unable to handle the message. Type /help to see the usage.";
const START_MESSAGE: &str = "Hi, I'm PayScribe!";

/* Invalid state.
 */
pub async fn invalid_state(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, INVALID_STATE_MESSAGE).await?;
    dialogue.exit().await?;
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, START_MESSAGE).await?;
    dialogue.exit().await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    dialogue.exit().await?;
    Ok(())
}
