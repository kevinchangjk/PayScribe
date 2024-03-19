use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use crate::bot::{handler::utils::display_debts, processor::add_payment};

use super::{
    super::dispatcher::{HandlerResult, State, UserDialogue},
    general::UNKNOWN_ERROR_MESSAGE,
    utils::{display_balances, make_keyboard, parse_amount, parse_debts, parse_username},
};

/* Utilities */
const HEADER_MESSAGE: &str = "Adding a new entry to pay back!\n\n";
const FOOTER_MESSAGE: &str = "Enter /cancel at any time to cancel the entry.\n\n";
const DEBT_INSTRUCTIONS_MESSAGE: &str =
    "Enter the usernames and the amounts as follows: \n\n@user1 amount1, @user2 amount2, etc.\n\n";

#[derive(Clone, Debug)]
pub struct PayBackParams {
    chat_id: String,
    sender_id: String,
    sender_username: Option<String>,
    datetime: String,
    total: f64,
    debts: Vec<(String, f64)>,
}

/* Action handler functions */

/* Handles a repeated call to pay back.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_pay_back(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are already paying back! Please complete or cancel the current operation before starting a new one.",
    ).await?;
    Ok(())
}

/* Cancels the pay back operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_pay_back(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Pay back cancelled!").await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_pay_back(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are currently paying back! Please complete or cancel the current payment before starting another command.",
    ).await?;
    Ok(())
}

/* Adds a pay back entry.
 * Entrypoint to the dialogue sequence.
 */
pub async fn action_pay_back(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!(
            "{HEADER_MESSAGE}Who did you pay back, and how much was it?\n{DEBT_INSTRUCTIONS_MESSAGE}{FOOTER_MESSAGE}"
        ),
    )
    .await?;
    dialogue.update(State::PayBackDebts).await?;
    Ok(())
}

/* Adds a pay back entry.
 * Bot receives a string representing debts, and proceeds to ask for confirmation.
 */
pub async fn action_pay_back_debts(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    Ok(())
}

/* Adds a pay back entry.
 * Bot receives a callback query for button menu, and responds accordingly.
 * If cancel, calls cancel handler. If edit, sends message and returns to previous state.
 * If confirm, calls processor.
 */
pub async fn action_pay_back_confirm(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    Ok(())
}
