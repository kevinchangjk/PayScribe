use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

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
        bot.send_message(msg.chat.id, format!("Sorry, I'm not intelligent enough to process that! 🤖\nPlease refer to {COMMAND_HELP} on how to use me!")).await?;
        Ok(())
    }
}

/* Invalid message during callback expected.
 */
pub async fn callback_invalid_message(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Hey, you don't have to text me...🗿\nJust click on any of the buttons above 👆 to continue!",
    )
    .await?;
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, format!("👋 Hi, I'm PayScribe!\n\nEnter {COMMAND_HELP} to check out the various commands I can assist you with, and let's get straight into tracking payments together!")).await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> HandlerResult {
    let mut commands = Command::descriptions().to_string();
    commands = commands.replace("–", "\\—");

    bot.send_message(
        msg.chat.id,
        format!("👋 Hey there\\! Need some help?\n\n_PayScribe_ is a handy assistant for keeping track of group payments\\. All you have to do is let me record down your group payments, and I'll simplify your debts to keep you updated with how much everyone owes one another\\.\n\nTo begin, you can add new payment records with {COMMAND_ADD_PAYMENT}\\. When specifying how much each person owes you can:\n\\- Divide the total cost equally among some users \\(e\\.g\\. sharing ticket prices equally among friends\\)\n\\- Specify the exact amount each person owes\n\\- Provide a ratio of how much each person owes \\(e\\.g\\. sharing subscription costs among friends with different usage durations\\)\n\nUse {COMMAND_VIEW_BALANCES} at any time to see how much everyone owes one another\\. To edit or delete payments, use {COMMAND_VIEW_PAYMENTS}, then {COMMAND_EDIT_PAYMENT} or {COMMAND_DELETE_PAYMENT}\\.\n\nAfter paying back your friends, be sure to record those down with the {COMMAND_PAY_BACK} command\\! \n\n*Check out all my commands here*:\n\n{}", commands),
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    Ok(())
}

/* Cancel command.
 * Called when state is at start, thus nothing to cancel.
 */
pub async fn action_cancel(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "I'm not doing anything right now. There's nothing to cancel! 👀",
    )
    .await?;
    Ok(())
}
