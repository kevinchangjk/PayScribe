use teloxide::{prelude::*, utils::command::BotCommands};

use crate::bot::dispatcher::Command;

use super::utils::{
    HandlerResult, COMMAND_ADD_PAYMENT, COMMAND_DELETE_PAYMENT, COMMAND_EDIT_PAYMENT, COMMAND_HELP,
    COMMAND_PAY_BACK, COMMAND_VIEW_BALANCES, COMMAND_VIEW_PAYMENTS,
};

/* Invalid state.
 */
pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    // Checks if msg is a service message, ignores it if so
    let is_service_msg = msg.from().is_none();

    if is_service_msg {
        Ok(())
    } else {
        bot.send_message(msg.chat.id, format!("Sorry, I'm not intelligent enough to process that! Please refer to {COMMAND_HELP} on how to use me!")).await?;
        Ok(())
    }
}

/* Invalid message during callback expected.
 */
pub async fn callback_invalid_message(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Hey, you don't have to text me...\nJust click on any of the buttons above to continue!",
    )
    .await?;
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, format!("Hi, I'm PayScribe!\n\nEnter {COMMAND_HELP} to check out the various commands I can assist you with, and let's get straight into tracking payments together!")).await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!("Hey there! Need some help?\n\nTo begin, you can add new payment records with {COMMAND_ADD_PAYMENT}. Use {COMMAND_VIEW_BALANCES} at any time to see how much everyone owes one another. \n\nTo edit or delete payment records, use {COMMAND_VIEW_PAYMENTS}, then {COMMAND_EDIT_PAYMENT} or {COMMAND_DELETE_PAYMENT} followed by the chosen serial no. of the record. After you have paid back your friends, be sure to record those down with the {COMMAND_PAY_BACK} command too! \n\nCheck out the full list of commands here:\n\n{}", Command::descriptions().to_string()),
    )
    .await?;
    Ok(())
}

/* Cancel command.
 * Called when state is at start, thus nothing to cancel.
 */
pub async fn action_cancel(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "I'm not doing anything right now. There's nothing to cancel!",
    )
    .await?;
    Ok(())
}
