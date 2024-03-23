use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    dispatcher::{HandlerResult, State, UserDialogue},
    handler::{
        general::{NO_TEXT_MESSAGE, UNKNOWN_ERROR_MESSAGE},
        utils::{
            display_balances, display_debts, make_keyboard, parse_amount, parse_username,
            process_debts,
        },
    },
    processor::add_payment,
    BotError,
};

/* Utilities */
const HEADER_MESSAGE: &str = "Adding a new payment entry!\n\n";
const FOOTER_MESSAGE: &str = "\n\n";

#[derive(Clone, Debug)]
pub struct DeletePaymentParams {
    payment_id: String,
    chat_id: String,
    datetime: String,
    description: Option<String>,
    creditor: Option<String>,
    total: Option<f64>,
    debts: Option<Vec<(String, f64)>>,
}

/* Action handler functions */

/* Handles a repeated call to delete payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are already deleting a payment entry! Please complete or cancel the current operation before starting a new one.",
    ).await?;
    Ok(())
}

/* Cancels the edit payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_delete_payment(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    bot.send_message(msg.chat.id, "Payment deletion cancelled, no changes made!")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "You are currently deleting a payment entry! Please complete or cancel the current payment entry before starting another command.",
    ).await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to delete payment without first viewing anything.
 */
pub async fn no_delete_payment(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please view the payment records first with /viewpayments!",
    )
    .await?;
    Ok(())
}

/* Edits a specified payment.
 * Bot will ask user to choose which part to edit, and ask for new values,
 * before confirming the changes and updating the balances.
 */
pub async fn action_delete_payment(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    Ok(())
}

pub async fn action_delete_payment_confirm(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    Ok(())
}
