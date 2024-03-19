use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use crate::bot::{handler::utils::display_debts, processor::add_payment, BotError};

use super::{
    super::dispatcher::{HandlerResult, State, UserDialogue},
    general::{NO_TEXT_MESSAGE, UNKNOWN_ERROR_MESSAGE},
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
    sender_username: String,
    datetime: String,
    total: f64,
    debts: Vec<(String, f64)>,
}

fn display_pay_back_entry(payment: &PayBackParams) -> String {
    format!(
        "You paid the following amounts to:\n{}",
        display_debts(&payment.debts)
    )
}

/* Displays an overview of the pay back entry, with a keyboard button menu.
 */
async fn display_pay_back_overview(
    bot: Bot,
    dialogue: UserDialogue,
    payment: PayBackParams,
) -> HandlerResult {
    let buttons = vec!["Cancel", "Edit", "Confirm"];
    let keyboard = make_keyboard(buttons, Some(3));

    bot.send_message(payment.chat_id.clone(), display_pay_back_entry(&payment))
        .reply_markup(keyboard)
        .await?;
    dialogue.update(State::PayBackConfirm { payment }).await?;
    Ok(())
}

async fn call_processor_pay_back(
    bot: Bot,
    dialogue: UserDialogue,
    payment: PayBackParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(Message { id, chat, .. }) = query.message {
        let payment_clone = payment.clone();
        let payment_overview = display_pay_back_entry(&payment);
        let description = format!("{} paid back", payment.sender_username);
        let total = payment.debts.iter().fold(0.0, |curr, next| curr + next.1);

        let updated_balances = add_payment(
            payment.chat_id,
            payment.sender_username.clone(),
            payment.sender_id,
            payment.datetime,
            &description,
            &payment.sender_username,
            total,
            payment.debts,
        );

        match updated_balances {
            Err(err) => {
                log::error!(
                    "Pay Back Submission - Processor failed to update balances for user {} in chat {} with payment {:?}: {}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone,
                    err.to_string()
                );
                bot.edit_message_text(
                    chat.id,
                    id,
                    format!("{}\nPay back failed!", err.to_string()),
                )
                .await?;
            }
            Ok(balances) => {
                log::info!(
                    "Pay Back Submission - Processor updated balances successfully for user {} in chat {}: {:?}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone
                );
                bot.edit_message_text(
                    chat.id,
                    id,
                    format!(
                        "Pay back added!\n\n{}Current balances:\n{}",
                        payment_overview,
                        display_balances(&balances)
                    ),
                )
                .await?;
            }
        }
        dialogue.exit().await?;
    }
    Ok(())
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
    match msg.text() {
        Some(text) => {
            let debts = parse_debts(text);
            if let Err(err) = debts {
                log::error!(
                    "Pay Back - Debt parsing failed for user {} in chat {}: {}",
                    msg.from().as_ref().unwrap().id,
                    msg.chat.id,
                    err.to_string()
                );
                bot.send_message(msg.chat.id, format!("{}\n\nWho did you pay back, and how much was it?\n{DEBT_INSTRUCTIONS_MESSAGE}{FOOTER_MESSAGE}", err.to_string())).await?;
                return Ok(());
            }

            let debts = debts?;
            let total = debts.iter().fold(0.0, |curr, next| curr + next.1);
            if let Some(user) = msg.from() {
                let username = match &user.username {
                    Some(user) => parse_username(user),
                    None => {
                        return Err(BotError::UserError(format!(
                            "Pay Back - Username not found for user ID {}",
                            user.id.to_string()
                        )));
                    }
                };
                let payment = PayBackParams {
                    chat_id: msg.chat.id.to_string(),
                    sender_id: msg.from().as_ref().unwrap().id.to_string(),
                    sender_username: username,
                    datetime: msg.date.to_string(),
                    total,
                    debts,
                };
                log::info!(
                    "Pay Back - Debt updated successfully for user {} in chat {}: {:?}",
                    payment.sender_id,
                    payment.chat_id,
                    payment
                );
                display_pay_back_overview(bot, dialogue, payment).await?;
            }
        }
        None => {
            bot.send_message(
                msg.chat.id,
                format!("{NO_TEXT_MESSAGE}{DEBT_INSTRUCTIONS_MESSAGE}{FOOTER_MESSAGE}"),
            )
            .await?;
        }
    }
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
    payment: PayBackParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(format!("{}", query.id)).await?;

        match button.as_str() {
            "Cancel" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(chat.id, id, "Pay back entry cancelled!")
                        .await?;
                    dialogue.exit().await?;
                }
            }
            "Edit" => {
                bot.send_message(
                    payment.chat_id,
                    format!(
                        "{HEADER_MESSAGE}Who did you pay back, and how much was it?\n\n{DEBT_INSTRUCTIONS_MESSAGE}{FOOTER_MESSAGE}"
                        ),
                    )
                    .await?;
                dialogue.update(State::PayBackDebts).await?;
            }
            "Confirm" => {
                call_processor_pay_back(bot, dialogue, payment, query).await?;
            }
            _ => {
                log::error!("Pay Back Confirm - Invalid button for user {} in chat {} with payment {:?}: {}",
                            payment.sender_id, payment.chat_id, payment, button);
            }
        }
    }
    Ok(())
}
