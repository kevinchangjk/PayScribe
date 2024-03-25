use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    processor::add_payment,
    {
        dispatcher::State,
        handler::utils::{
            display_balances, display_debts, make_keyboard, parse_debts, parse_username, BotError,
            HandlerResult, UserDialogue, DEBT_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE,
        },
    },
};

/* Utilities */
#[derive(Clone, Debug)]
pub struct PayBackParams {
    chat_id: String,
    sender_id: String,
    sender_username: String,
    datetime: String,
    total: f64,
    debts: Vec<(String, f64)>,
}

const CANCEL_MESSAGE: &str =
    "👌 Sure, I've cancelled adding the payment. No changes have been made!";

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
        let description = format!("{} paid back!", payment.sender_username);
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
                    format!(
                        "❓ Hmm, something went wrong! Sorry, I can't add the payment right now."
                    ),
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
                        "🎉 I've added the payment!\n\n{}\nHere are the updated balances:\n{}",
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
        "🚫 You are already paying back! Please complete or cancel the current operation before starting a new one.",
    ).await?;
    Ok(())
}

/* Cancels the pay back operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_pay_back(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, CANCEL_MESSAGE).await?;
    dialogue.exit().await?;
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_pay_back(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "🚫 You are currently paying back! Please complete or cancel the current payment before starting another command.",
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
            "Alright! Who did you pay back, and how much did you pay?\n{DEBT_INSTRUCTIONS_MESSAGE}"
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
                bot.send_message(msg.chat.id, err.to_string()).await?;
                return Ok(());
            }

            let debts = debts?;
            let total = debts.iter().fold(0.0, |curr, next| curr + next.1);
            if let Some(user) = msg.from() {
                if let Some(username) = &user.username {
                    let username = parse_username(username);
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
        }
        None => {
            bot.send_message(msg.chat.id, format!("{NO_TEXT_MESSAGE}"))
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
        bot.answer_callback_query(query.id.to_string()).await?;

        match button.as_str() {
            "Cancel" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(chat.id, id, CANCEL_MESSAGE).await?;
                    dialogue.exit().await?;
                }
            }
            "Edit" => {
                bot.send_message(
                    payment.chat_id,
                    format!(
                        "Alright! Who did you pay back, and how much did you pay?\n\n{DEBT_INSTRUCTIONS_MESSAGE}"
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
