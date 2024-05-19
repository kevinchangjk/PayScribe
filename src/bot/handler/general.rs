use teloxide::{prelude::*, types::ParseMode, utils::command::BotCommands};

use crate::bot::dispatcher::Command;

use super::{
    constants::{
        COMMAND_ADD_PAYMENT, COMMAND_BALANCES, COMMAND_DELETE_PAYMENT, COMMAND_EDIT_PAYMENT,
        COMMAND_HELP, COMMAND_PAY_BACK, COMMAND_SPENDINGS, COMMAND_VIEW_PAYMENTS, FEEDBACK_URL,
        USER_GUIDE_URL,
    },
    utils::{assert_handle_request_limit, send_bot_message, HandlerResult},
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
        // send_bot_message(&bot, &msg, format!("Sorry, I'm not intelligent enough to process that! 🤖\nPlease refer to {COMMAND_HELP} on how to use me!")).await?;
        Ok(())
    }
}

/* Invalid message during callback expected.
 * Currently, simply does not respond to anything. Reduces spam.
 */
pub async fn callback_invalid_message(_bot: Bot, _msg: Message) -> HandlerResult {
    /*
    send_bot_message(
        &bot,
        &msg
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
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let introduction = format!("👋 Hello! I'm PayScribe! 😊\n\n🧚 I'll be tracking your group payments and working my magic to simplify your debts, so you won't have to juggle so many payments back to your friends!");
    let add_info = &format!("✍️ Ready to track together in this group chat? Start with {COMMAND_ADD_PAYMENT}! You can {COMMAND_VIEW_PAYMENTS} anytime, and I'll help to {COMMAND_EDIT_PAYMENT} or {COMMAND_DELETE_PAYMENT} if you'd like!");
    let view_info = &format!("🙈 Check out {COMMAND_SPENDINGS} to see who's been splurging! Peek at {COMMAND_BALANCES} for who owes what, but don't forget to {COMMAND_PAY_BACK} your friends!");
    let closing =
        &format!("🤗 Have fun tracking, and don't hesitate to ask me for {COMMAND_HELP} anytime!");
    send_bot_message(
        &bot,
        &msg,
        format!("{introduction}\n\n{add_info}\n\n{view_info}\n\n{closing}"),
    )
    .await?;
    Ok(())
}

/* Help command.
 * Displays a list of commands available to the user.
 */
pub async fn action_help(bot: Bot, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let mut commands = Command::descriptions().to_string();
    commands = commands.replace("–", "\\—");

    let user_guide_info = &format!("🆘 For all the nitty\\-gritty details on supported time zones, currencies, and more, check out my [User Guide]({USER_GUIDE_URL})\\!");
    let feedback_info = &format!("💡 And if you have any feedback for me, I'd love to hear it over [here]({FEEDBACK_URL})\\!");

    send_bot_message(
        &bot,
        &msg,
        format!(
            "⭐️ *My Commands* ⭐️\n\n{}\n\n{user_guide_info}\n\n{feedback_info}",
            commands
        ),
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;

    Ok(())
}

/* Cancel command.
 * Called when state is at start, thus nothing to cancel.
 */
pub async fn action_cancel(bot: Bot, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    send_bot_message(
        &bot,
        &msg,
        format!("I'm not doing anything... 👀\nThere's nothing to cancel!"),
    )
    .await?;
    Ok(())
}
