use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

use crate::bot::dispatcher::Command;

use super::{
    constants::{
        COMMAND_ADD_PAYMENT, COMMAND_BALANCES, COMMAND_DELETE_PAYMENT, COMMAND_EDIT_PAYMENT,
        COMMAND_HELP, COMMAND_PAY_BACK, COMMAND_SETTINGS, COMMAND_VIEW_PAYMENTS, USER_GUIDE_URL,
    },
    utils::HandlerResult,
};

/* Invalid state.
 * This action is invoked when the bot is in start state, and there is a non-command message
 * addressed to it.
 * Currently, simply does not respond to anything. Reduces spam.
 */
pub async fn invalid_state(_bot: Bot, msg: Message) -> HandlerResult {
    // Checks if msg is a service message, ignores it if so
    let is_service_msg = msg.from().is_none();

    if is_service_msg {
        Ok(())
    } else {
        // bot.send_message(msg.chat.id, format!("Sorry, I'm not intelligent enough to process that! 🤖\nPlease refer to {COMMAND_HELP} on how to use me!")).await?;
        Ok(())
    }
}

/* Invalid message during callback expected.
 */
pub async fn callback_invalid_message(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Hey, you don't have to text me...\nJust click on any of the buttons above 👆 to continue!",
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

    let introduction = "👋 Hey there\\! Need some help?\n\n_PayScribe_ is a handy assistant for keeping track of group payments, and simplifying your debts to keep you updated with how much everyone owes one another\\.";
    let add_info = &format!("To begin, you can add new payment records with {COMMAND_ADD_PAYMENT}\\. When splitting the total amount, you can:\n\\- Divide the total cost equally \\(e\\.g\\. sharing ticket prices equally among friends\\)\n\\- Specify the exact amount for each person\n\\- Provide a proportion of how much each person owes \\(e\\.g\\. splitting the electricity bill 40\\-60\\)");
    let view_info = &format!("Use {COMMAND_BALANCES} see how much everyone owes one another\\. To edit or delete payments, use {COMMAND_VIEW_PAYMENTS}, then {COMMAND_EDIT_PAYMENT} or {COMMAND_DELETE_PAYMENT}\\.");
    let payback_info = &format!("After paying back your friends, be sure to record those down with the {COMMAND_PAY_BACK} command\\!");
    let settings_info = &format!("With {COMMAND_SETTINGS}, you can configure settings for the chat\\. You can find all supported time zones, currencies, along with other useful details in my [User Guide]({USER_GUIDE_URL})\\!");

    bot.send_message(
        msg.chat.id,
        format!("{introduction}\n\n{add_info}\n\n{view_info}\n\n{payback_info}\n\n{settings_info}\n\n*All commands*:\n\n{}", commands),
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
