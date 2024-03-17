use teloxide::{prelude::*, utils::command::BotCommands};

use super::super::dispatcher::{Command, HandlerResult, UserDialogue};

/* Invalid state.
 */
pub async fn invalid_state(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    dialogue.exit().await?;
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Hi, I'm PayScribe!").await?;
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
