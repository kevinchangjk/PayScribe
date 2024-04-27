use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

use crate::bot::dispatcher::Command;

use super::{
    constants::{
        COMMAND_ADD_PAYMENT, COMMAND_BALANCES, COMMAND_DELETE_PAYMENT, COMMAND_EDIT_PAYMENT,
        COMMAND_HELP, COMMAND_PAY_BACK, COMMAND_SETTINGS, COMMAND_SPENDINGS, COMMAND_VIEW_PAYMENTS,
        USER_GUIDE_URL,
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
 * Currently, simply does not respond to anything. Reduces spam.
 */
pub async fn callback_invalid_message(_bot: Bot, _msg: Message) -> HandlerResult {
    /*
    bot.send_message(
        msg.chat.id,
        "Hey, you don't have to text me...\nJust click on any of the buttons above 👆 to continue!",
    )
    .await?;
    */
    Ok(())
}

/* Start command.
 * Displays a welcome message to the user.
 */
pub async fn action_start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, format!("👋 Hello! I'm PayScribe! 😊\n\nJust type {COMMAND_HELP} to see everything I can help you with, and let's dive right into tracking payments together!")).await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> HandlerResult {
    let mut commands = Command::descriptions().to_string();
    commands = commands.replace("–", "\\—");

    let introduction = "👋 Hello\\! Need a hand? 😉\n\n_PayScribe_ is your handy assistant for tracking group payments\\! Plus, I'll work my magic to simplify your debts, so you won't have to juggle so many payments back to your friends\\!";
    let add_info = &format!("✍️ Ready to start tracking? You can add new payment records with {COMMAND_ADD_PAYMENT}\\! When it comes to splitting the total, you can choose between:\n\\- Dividing it equally\n\\- Entering the exact amount for each person\n\\- Specifying the proportion of the total owed for each person");
    let view_info = &format!("🙈 Use {COMMAND_BALANCES} to peek at who owes what, and {COMMAND_SPENDINGS} to see who's been splurging\\! If you need to edit any records, just start with {COMMAND_VIEW_PAYMENTS}, then try {COMMAND_EDIT_PAYMENT} or {COMMAND_DELETE_PAYMENT}\\!");
    let payback_info = &format!("💸 Once you've paid back your friends, don't forget to jot it down with {COMMAND_PAY_BACK}\\!");
    let settings_info = &format!("⚙️ Lastly, I've got some group settings you can tweak with {COMMAND_SETTINGS}\\! For all the nitty\\-gritty details on supported time zones, currencies, and more, check out my [User Guide]({USER_GUIDE_URL})\\!");

    bot.send_message(
        msg.chat.id,
        format!("{introduction}\n\n{add_info}\n\n{view_info}\n\n{payback_info}\n\n{settings_info}\n\n🌟 *My Commands* 🌟\n\n{}", commands),
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
        "I'm not doing anything right now... 👀 There's nothing to cancel!",
    )
    .await?;
    Ok(())
}
