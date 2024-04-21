use teloxide::{payloads::SendMessageSetters, prelude::*, types::Message};

use crate::bot::{
    currency::{get_default_currency, CURRENCY_DEFAULT},
    dispatcher::State,
    handler::{
        constants::{
            COMMAND_CANCEL, COMMAND_HELP, CURRENCY_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE,
            PAY_BACK_INSTRUCTIONS_MESSAGE, UNKNOWN_ERROR_MESSAGE,
        },
        utils::{
            display_balances, display_debts, display_username, get_chat_default_currency,
            get_currency, make_keyboard, parse_debts_payback, parse_username, use_currency,
            Currency, HandlerResult, UserDialogue,
        },
    },
    processor::add_payment,
};

/* Utilities */
#[derive(Clone, Debug)]
pub struct PayBackParams {
    chat_id: String,
    sender_id: String,
    sender_username: String,
    datetime: String,
    currency: Currency,
    total: i64,
    debts: Vec<(String, i64)>,
}

const CANCEL_MESSAGE: &str =
    "Sure, I've cancelled adding the payment. No changes have been made! 👌";

fn display_pay_back_entry(payment: &PayBackParams) -> String {
    let currency_info: String;
    let actual_currency = use_currency(payment.currency.clone(), &payment.chat_id);
    if actual_currency.0 == CURRENCY_DEFAULT.0 {
        currency_info = "".to_string();
    } else {
        currency_info = format!("in {} ", actual_currency.0);
    }

    format!(
        "You paid the following amounts {}to:\n{}",
        currency_info,
        display_debts(&payment.debts, actual_currency.1)
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
        let description = format!("{} paid back!", display_username(&payment.sender_username));

        let updated_balances = add_payment(
            payment.chat_id,
            payment.sender_username.clone(),
            payment.sender_id,
            payment.datetime,
            &description,
            &payment.sender_username,
            &payment.currency.0,
            payment.total,
            payment.debts,
        )
        .await;

        match updated_balances {
            Err(err) => {
                bot.edit_message_text(
                    chat.id,
                    id,
                    format!(
                        "❓ Hmm, something went wrong! Sorry, I can't add the payment right now."
                    ),
                )
                .await?;

                // Logging
                log::error!(
                    "Pay Back Submission - Processor failed to update balances for user {} in chat {} with payment {:?}: {}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone,
                    err.to_string()
                    );
            }
            Ok(balances) => {
                bot.edit_message_text(
                    chat.id,
                    id,
                    format!(
                        "🎉 I've added the payment! 🎉\n\n{}\nHere are the updated balances:\n{}",
                        payment_overview,
                        display_balances(&balances, &chat.id.to_string())
                    ),
                )
                .await?;

                // Logging
                log::info!(
                    "Pay Back Submission - Processor updated balances successfully for user {} in chat {}: {:?}",
                    payment_clone.sender_id,
                    payment_clone.chat_id,
                    payment_clone
                    );
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
        format!("🚫 You are already paying back! Please complete or {COMMAND_CANCEL} the current operation before starting a new one."),
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
        format!("🚫 You are currently paying back! Please complete or {COMMAND_CANCEL} the current payment before starting another command."),
        ).await?;
    Ok(())
}

/* Adds a pay back entry.
 * Entrypoint to the dialogue sequence.
 */
pub async fn action_pay_back(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    let buttons = vec!["Cancel", "Skip", "Set Currency"];
    let keyboard = make_keyboard(buttons, Some(3));
    bot.send_message(
        msg.chat.id,
        format!("Alright! What currency did you pay in? You can also choose to skip and not enter a currency."),
        )
        .reply_markup(keyboard)
        .await?;
    dialogue.update(State::PayBackCurrencyMenu).await?;
    Ok(())
}

/* Adds a pay back entry.
 * Bot receives a callback query indicating to skip or add currency.
 */
pub async fn action_pay_back_currency_menu(
    bot: Bot,
    dialogue: UserDialogue,
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
            "Skip" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Sure! Who did you pay back and how much did you pay?\n\n{PAY_BACK_INSTRUCTIONS_MESSAGE}"
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::PayBackDebts {
                            currency: get_default_currency(),
                        })
                        .await?;
                }
            }
            "Set Currency" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Sure! What currency did you pay in?\n\n{CURRENCY_INSTRUCTIONS_MESSAGE}"
                            ),
                    )
                    .await?;
                    dialogue.update(State::PayBackCurrency).await?;
                }
            }
            _ => {
                if let Some(msg) = query.message {
                    if let Some(user) = msg.from() {
                        log::error!(
                            "Pay Back Currency Menu - Invalid button for user {} in chat {}: {}",
                            user.id,
                            msg.chat.id,
                            button
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/* Adds a pay back entry.
 * Bot receives either a string representing a currency code.
 */
pub async fn action_pay_back_currency(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let currency_code = text.to_uppercase();
            let currency = get_currency(&currency_code);
            match currency {
                Ok(currency) => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Sure! We're working with {}.\n\nWho did you pay back and how much did you pay?\n{PAY_BACK_INSTRUCTIONS_MESSAGE}",
                            currency_code
                            ),
                            ).await?;
                    dialogue.update(State::PayBackDebts { currency }).await?;
                }
                Err(err) => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "{} You can check out the supported currencies in the User Guide with {COMMAND_HELP}.",
                            err.to_string()
                        ),
                    )
                    .await?;
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
 * Bot receives a string representing debts, and proceeds to ask for confirmation.
 */
pub async fn action_pay_back_debts(
    bot: Bot,
    dialogue: UserDialogue,
    currency: Currency,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    match msg.text() {
        Some(text) => {
            if let Some(user) = msg.from() {
                if let Some(username) = &user.username {
                    let username = parse_username(username);
                    if let Err(err) = &username {
                        bot.send_message(msg.chat.id, UNKNOWN_ERROR_MESSAGE).await?;

                        // Logging
                        log::error!(
                            "Pay Back Debts - Failed to parse username for sender {}: {}",
                            user.id,
                            err.to_string()
                        );
                    }
                    let username = username?;

                    let actual_currency: Currency;
                    if currency.0 == CURRENCY_DEFAULT.0 {
                        actual_currency = get_chat_default_currency(&chat_id);
                    } else {
                        actual_currency = currency.clone();
                    }

                    let debts = parse_debts_payback(text, actual_currency.clone(), &username);
                    if let Err(err) = debts {
                        bot.send_message(
                            chat_id,
                            format!("{}\n\n{PAY_BACK_INSTRUCTIONS_MESSAGE}", err.to_string()),
                        )
                        .await?;
                        return Ok(());
                    }

                    let debts = debts?;
                    let total = debts.iter().fold(0, |curr, next| curr + next.1);
                    let payment = PayBackParams {
                        chat_id,
                        sender_id: msg.from().as_ref().unwrap().id.to_string(),
                        sender_username: username,
                        datetime: msg.date.to_string(),
                        currency,
                        total,
                        debts,
                    };
                    display_pay_back_overview(bot, dialogue, payment).await?;
                }
            }
        }
        None => {
            bot.send_message(chat_id, format!("{NO_TEXT_MESSAGE}"))
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
                if let Some(Message { id, chat, .. }) = query.message {
                    let buttons = vec!["Cancel", "Skip", "Set Currency"];
                    let keyboard = make_keyboard(buttons, Some(3));
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Alright! What currency did you pay in? You can also choose to skip and not enter a currency."
                            ),
                            )
                        .reply_markup(keyboard)
                        .await?;
                    dialogue.update(State::PayBackCurrencyMenu).await?;
                }
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
