use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{Message, MessageId},
};

use crate::bot::{
    currency::{get_default_currency, Currency, CURRENCY_DEFAULT},
    dispatcher::State,
    handler::{
        constants::{
            COMMAND_CANCEL, COMMAND_HELP, CURRENCY_INSTRUCTIONS_MESSAGE, NO_TEXT_MESSAGE,
            PAY_BACK_INSTRUCTIONS_MESSAGE, UNKNOWN_ERROR_MESSAGE,
        },
        utils::{
            display_balance_header, display_balances, display_debts, display_username,
            get_chat_default_currency, get_currency, make_keyboard, parse_debts_payback,
            parse_username, use_currency, HandlerResult, UserDialogue,
        },
    },
    processor::add_payment,
};

use super::utils::{
    assert_handle_request_limit, delete_bot_messages, is_erase_messages, send_bot_message,
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
    "Okay! I've cancelled adding the payment. No changes have been made! 🌟";

// Controls the state for misc handler actions that return to same state.
async fn repeat_state(
    dialogue: UserDialogue,
    state: State,
    new_message: MessageId,
) -> HandlerResult {
    match state {
        State::PayBackCurrencyMenu { mut messages } => {
            messages.push(new_message);
            dialogue
                .update(State::PayBackCurrencyMenu { messages })
                .await?;
        }
        State::PayBackCurrency { mut messages } => {
            messages.push(new_message);
            dialogue.update(State::PayBackCurrency { messages }).await?;
        }
        State::PayBackDebts {
            mut messages,
            currency,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::PayBackDebts { messages, currency })
                .await?;
        }
        State::PayBackConfirm {
            mut messages,
            payment,
        } => {
            messages.push(new_message);
            dialogue
                .update(State::PayBackConfirm { messages, payment })
                .await?;
        }
        _ => (),
    }
    Ok(())
}

// Controls the dialogue for ending a pay back operation.
async fn complete_pay_back(
    bot: &Bot,
    dialogue: UserDialogue,
    chat_id: &str,
    messages: Vec<MessageId>,
) -> HandlerResult {
    if is_erase_messages(chat_id) {
        delete_bot_messages(&bot, chat_id, messages).await?;
    }
    dialogue.exit().await?;
    Ok(())
}

fn display_pay_back_entry(payment: &PayBackParams) -> String {
    let currency_info: String;
    let actual_currency = use_currency(payment.currency.clone(), &payment.chat_id);
    if actual_currency.0 == CURRENCY_DEFAULT.0 {
        currency_info = "".to_string();
    } else {
        currency_info = format!(" in {} ", actual_currency.0);
    }

    format!(
        "You've paid{}:\n{}",
        currency_info,
        display_debts(&payment.debts, actual_currency.1)
    )
}

/* Displays an overview of the pay back entry, with a keyboard button menu.
*/
async fn display_pay_back_overview(
    bot: &Bot,
    msg: &Message,
    dialogue: &UserDialogue,
    mut messages: Vec<MessageId>,
    payment: PayBackParams,
) -> HandlerResult {
    let buttons = vec!["Cancel", "Edit", "Confirm"];
    let keyboard = make_keyboard(buttons, Some(2));

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!(
            "Amazing! 💫 Did I get everything right?\n\n{}",
            display_pay_back_entry(&payment)
        ),
    )
    .reply_markup(keyboard)
    .await?
    .id;
    messages.push(new_message);
    dialogue
        .update(State::PayBackConfirm { messages, payment })
        .await?;
    Ok(())
}

async fn call_processor_pay_back(
    bot: Bot,
    dialogue: UserDialogue,
    messages: Vec<MessageId>,
    payment: PayBackParams,
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(msg) = query.message {
        let chat_id = msg.chat.id;
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
                send_bot_message(
                    &bot,
                    &msg,
                    format!(
                        "⁉️ Oh no! Something went wrong! 🥺 I'm sorry, but I can't add the payment right now. Please try again later!\n\n"
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
                send_bot_message(
                    &bot,
                    &msg,
                    format!("🎉 Yay! Payment added! 🎉\n\n{}", payment_overview,),
                )
                .await?;
                send_bot_message(
                    &bot,
                    &msg,
                    format!(
                        "{}{}",
                        display_balance_header(&chat_id.to_string(), &payment.currency.0),
                        display_balances(&balances)
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
        complete_pay_back(&bot, dialogue, &chat_id.to_string(), messages).await?;
    }
    Ok(())
}

/* Action handler functions */

/* Handles a repeated call to pay back.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_pay_back(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("🚫 Oops! It seems like you're already in the middle of paying back! Please finish or {COMMAND_CANCEL} this before starting another one with me."),
        ).await?.id;

    repeat_state(dialogue, state, new_message).await?;
    Ok(())
}

/* Cancels the pay back operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_pay_back(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    send_bot_message(&bot, &msg, CANCEL_MESSAGE.to_string()).await?;

    match state {
        State::PayBackCurrencyMenu { messages }
        | State::PayBackCurrency { messages }
        | State::PayBackDebts { messages, .. }
        | State::PayBackConfirm { messages, .. } => {
            complete_pay_back(&bot, dialogue, &msg.chat.id.to_string(), messages).await?;
        }
        _ => (),
    }
    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of adding a payment.
 */
pub async fn block_pay_back(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    msg: Message,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("🚫 Oops! It seems like you're in the middle of paying back! Please finish or {COMMAND_CANCEL} this before starting something new with me."),
        ).await?.id;

    repeat_state(dialogue, state, new_message).await?;
    Ok(())
}

/* Adds a pay back entry.
 * Entrypoint to the dialogue sequence.
 */
pub async fn action_pay_back(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let buttons = vec!["Cancel", "Skip", "Set Currency"];
    let keyboard = make_keyboard(buttons, Some(2));
    let new_message = send_bot_message(
        &bot,
        &msg,
        format!("Absolutely! 🙌 Would you like to set a currency for this payment? You can also choose to skip this step."),
        )
        .reply_markup(keyboard)
        .await?.id;

    dialogue
        .update(State::PayBackCurrencyMenu {
            messages: vec![new_message],
        })
        .await?;
    Ok(())
}

/* Adds a pay back entry.
 * Bot receives a callback query indicating to skip or add currency.
 */
pub async fn action_pay_back_currency_menu(
    bot: Bot,
    dialogue: UserDialogue,
    state: State,
    query: CallbackQuery,
    messages: Vec<MessageId>,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        match button.as_str() {
            "Cancel" => {
                if let Some(msg) = query.message {
                    cancel_pay_back(bot, dialogue, state, msg).await?;
                }
            }
            "Skip" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Sure! Who and how much did you pay back?\n\n{PAY_BACK_INSTRUCTIONS_MESSAGE}"
                            ),
                            )
                        .await?;
                    dialogue
                        .update(State::PayBackDebts {
                            messages,
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
                    dialogue.update(State::PayBackCurrency { messages }).await?;
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
    state: State,
    msg: Message,
    mut messages: Vec<MessageId>,
) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let currency_code = text.to_uppercase();
            let currency = get_currency(&currency_code);
            match currency {
                Ok(currency) => {
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "{}, awesome! Who and how much did you pay back?\n\n{PAY_BACK_INSTRUCTIONS_MESSAGE}",
                            currency_code
                            ),
                            ).await?.id;
                    messages.push(new_message);
                    dialogue
                        .update(State::PayBackDebts { messages, currency })
                        .await?;
                }
                Err(err) => {
                    let new_message = send_bot_message(
                        &bot,
                        &msg,
                        format!(
                            "{}\n\n⭐️ If you're unsure of the currency code, you can always check out my User Guide with {COMMAND_HELP}.",
                            err.to_string()
                        ),
                    )
                    .await?.id;
                    repeat_state(dialogue, state, new_message).await?;
                }
            }
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, state, new_message).await?;
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
    state: State,
    msg: Message,
    (messages, currency): (Vec<MessageId>, Currency),
) -> HandlerResult {
    let chat_id = msg.chat.id.to_string();
    match msg.text() {
        Some(text) => {
            if let Some(user) = msg.from() {
                if let Some(username) = &user.username {
                    let username = parse_username(username);
                    if let Err(err) = &username {
                        let new_message =
                            send_bot_message(&bot, &msg, UNKNOWN_ERROR_MESSAGE.to_string())
                                .await?
                                .id;

                        // Logging
                        log::error!(
                            "Pay Back Debts - Failed to parse username for sender {}: {}",
                            user.id,
                            err.to_string()
                        );

                        repeat_state(dialogue, state, new_message).await?;
                        return Ok(());
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
                        let new_message = send_bot_message(
                            &bot,
                            &msg,
                            format!("{}\n\n{PAY_BACK_INSTRUCTIONS_MESSAGE}", err.to_string()),
                        )
                        .await?
                        .id;

                        repeat_state(dialogue, state, new_message).await?;
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
                    display_pay_back_overview(&bot, &msg, &dialogue, messages, payment).await?;
                }
            }
        }
        None => {
            let new_message = send_bot_message(&bot, &msg, format!("{NO_TEXT_MESSAGE}"))
                .await?
                .id;
            repeat_state(dialogue, state, new_message).await?;
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
    state: State,
    (messages, payment): (Vec<MessageId>, PayBackParams),
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        match button.as_str() {
            "Cancel" => {
                if let Some(msg) = query.message {
                    cancel_pay_back(bot, dialogue, state, msg).await?;
                }
            }
            "Edit" => {
                if let Some(Message { id, chat, .. }) = query.message {
                    let buttons = vec!["Cancel", "Skip", "Set Currency"];
                    let keyboard = make_keyboard(buttons, Some(2));
                    bot.edit_message_text(
                        chat.id,
                        id,
                        format!(
                            "Absolutely! 🙌 Would you like to set a currency for this payment? You can also choose to skip this step."
                            ),
                            )
                        .reply_markup(keyboard)
                        .await?;
                    dialogue
                        .update(State::PayBackCurrencyMenu { messages })
                        .await?;
                }
            }
            "Confirm" => {
                call_processor_pay_back(bot, dialogue, messages, payment, query).await?;
            }
            _ => {
                log::error!("Pay Back Confirm - Invalid button for user {} in chat {} with payment {:?}: {}",
                            payment.sender_id, payment.chat_id, payment, button);
            }
        }
    }
    Ok(())
}
