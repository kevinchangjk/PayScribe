use teloxide::{prelude::*, utils::command::BotCommands};

use super::super::dispatcher::{Command, HandlerResult, UserDialogue};

pub const UNKNOWN_ERROR_MESSAGE: &str = "An unknown error occurred. Please try again.";
pub const NO_TEXT_MESSAGE: &str = "Please reply in text.\n\n";
pub const DEBT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and the amounts as follows: \n\n@user1 amount1, @user2 amount2, etc.\n\n";
const INVALID_STATE_MESSAGE: &str = "Unable to handle the message. Type /help to see the usage.";
const START_MESSAGE: &str = "Hi, I'm PayScribe!\n\nEnter /help to check out the various commands I can assist with, and let's get straight into tracking payments together!";
const HELP_MESSAGE: &str = "Need some help?\n\nTo begin, you can add new payment records with /addpayment. Use /viewbalances at any time to see how much everyone owes one another.\n\nTo edit or delete payment records, use /viewpayments, then /editpayment or /deletepayment followed by the chosen serial no. of the record. After you have paid back your friends, be sure to record those down with the /payback command too!\n\nCheck out the full list of commands here:\n\n";

/* Invalid state.
 */
pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    // Checks if msg is a service message, ignores it if so
    let is_service_msg = msg.from().is_none();

    if is_service_msg {
        Ok(())
    } else {
        bot.send_message(msg.chat.id, INVALID_STATE_MESSAGE).await?;
        Ok(())
    }
}

/* Invalid message during callback expected.
 */
pub async fn callback_invalid_message(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You don't have to text me. Just click on any of the buttons above to continue!",
    )
    .await?;
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, START_MESSAGE).await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!("{HELP_MESSAGE}{}", Command::descriptions().to_string()),
    )
    .await?;
    Ok(())
}

/* Cancel command.
 * Called when state is at start, thus nothing to cancel.
 */
pub async fn action_cancel(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Nothing to cancel!").await?;
    Ok(())
}
