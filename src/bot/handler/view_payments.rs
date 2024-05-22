use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardMarkup, Message},
};

use crate::bot::{
    currency::{get_default_currency, Currency},
    dispatcher::State,
    handler::{
        constants::{COMMAND_ADD_PAYMENT, UNKNOWN_ERROR_MESSAGE},
        utils::{
            display_payment, get_currency, make_keyboard, retrieve_time_zone, send_bot_message,
            HandlerResult, UserDialogue,
        },
    },
    processor::{view_payments, ProcessError},
    redis::{CrudError, UserPayment},
};

use super::{
    action_delete_payment, action_edit_payment, block_delete_payment, block_edit_payment,
    cancel_delete_payment, cancel_edit_payment, handle_repeated_delete_payment,
    handle_repeated_edit_payment, utils::assert_handle_request_limit, SelectPaymentType,
};

/* Utilities */
const HEADER_MESSAGE_FRONT: &str = "Anytime! ☺️\nI've recorded ";
const HEADER_MESSAGE_BACK: &str = " payments. Here are the latest entries!\n\n";

#[derive(Clone, Debug)]
pub struct Payment {
    pub payment_id: String,
    pub chat_id: String,
    pub datetime: String,
    pub description: String,
    pub creditor: String,
    pub currency: Currency,
    pub total: i64,
    pub debts: Vec<(String, i64)>,
}

fn unfold_payment(payment: UserPayment) -> Payment {
    let currency = get_currency(&payment.payment.currency);
    match currency {
        Ok(currency) => Payment {
            payment_id: payment.payment_id,
            chat_id: payment.chat_id,
            datetime: payment.payment.datetime,
            description: payment.payment.description,
            creditor: payment.payment.creditor,
            currency,
            total: payment.payment.total,
            debts: payment.payment.debts,
        },
        Err(_) => Payment {
            payment_id: payment.payment_id,
            chat_id: payment.chat_id,
            datetime: payment.payment.datetime,
            description: payment.payment.description,
            creditor: payment.payment.creditor,
            currency: get_default_currency(),
            total: payment.payment.total,
            debts: payment.payment.debts,
        },
    }
}

fn display_payments_paged(payments: &Vec<Payment>, page: usize, chat_id: &str) -> String {
    let time_zone = retrieve_time_zone(chat_id);
    let start_index = page * 5;
    let displayed_payments: &[Payment];
    if start_index + 5 >= payments.len() {
        displayed_payments = &payments[start_index..];
    } else {
        displayed_payments = &payments[start_index..start_index + 5];
    }

    let serial_num = start_index + 1;
    let formatted_payments = displayed_payments
        .iter()
        .enumerate()
        .map(|(index, payment)| display_payment(payment, serial_num + index, time_zone));

    format!("{}", formatted_payments.collect::<Vec<String>>().join(""))
}

fn get_navigation_menu() -> InlineKeyboardMarkup {
    let buttons = vec!["Newer", "Older"];
    make_keyboard(buttons, Some(2))
}

fn get_select_menu(page: usize, payments: &Vec<Payment>) -> InlineKeyboardMarkup {
    let start_index = page * 5;
    let end_index = if start_index + 5 >= payments.len() {
        payments.len()
    } else {
        start_index + 5
    };

    let mut buttons: Vec<String> = (start_index..end_index)
        .map(|index| format!("{}", index + 1))
        .collect();
    buttons.push("Cancel".to_string());

    make_keyboard(
        buttons.iter().map(|option| option.as_str()).collect(),
        Some(3),
    )
}

/* Handles a repeated call to edit/delete payment entry.
 * Does nothing, simply notifies the user.
 */
pub async fn handle_repeated_select_payment(
    bot: Bot,
    msg: Message,
    (_payments, _page, function): (Vec<Payment>, usize, SelectPaymentType),
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    match function {
        SelectPaymentType::EditPayment => {
            handle_repeated_edit_payment(bot, msg).await?;
        }
        SelectPaymentType::DeletePayment => {
            handle_repeated_delete_payment(bot, msg).await?;
        }
    }
    Ok(())
}

/* Cancels the edit/delete payment operation.
 * Can be called at any step of the process.
 */
pub async fn cancel_select_payment(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    state: State,
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    if let State::SelectPayment {
        payments: _,
        page: _,
        ref function,
    } = state
    {
        match function {
            SelectPaymentType::EditPayment => {
                cancel_edit_payment(bot, dialogue, msg, state).await?;
            }
            SelectPaymentType::DeletePayment => {
                cancel_delete_payment(bot, dialogue, msg, state).await?;
            }
        }
    }

    Ok(())
}

/* Blocks user command.
 * Called when user attempts to start another operation in the middle of editing/deleting a payment.
 */
pub async fn block_select_payment(
    bot: Bot,
    msg: Message,
    (_payments, _page, function): (Vec<Payment>, usize, SelectPaymentType),
) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    match function {
        SelectPaymentType::EditPayment => {
            block_edit_payment(bot, msg).await?;
        }
        SelectPaymentType::DeletePayment => {
            block_delete_payment(bot, msg).await?;
        }
    }
    Ok(())
}

/* View all payments.
 * Bot retrieves all payments, and displays the most recent 5.
 * Then, presents a previous and next page button for the user to navigate the pagination.
 */
pub async fn action_view_payments(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    if !assert_handle_request_limit(msg.clone()) {
        return Ok(());
    }

    let chat_id = msg.chat.id.to_string();
    let user = msg.from();
    if let Some(user) = user {
        let sender_id = user.id.to_string();
        let sender_username = user.username.clone();
        let payments = view_payments(&chat_id, &sender_id, sender_username.as_deref());
        match payments {
            Ok(payments) => {
                let payments: Vec<Payment> = payments
                    .into_iter()
                    .map(|payment| unfold_payment(payment))
                    .collect();
                send_bot_message(
                    &bot,
                    &msg,
                    format!(
                        "{HEADER_MESSAGE_FRONT}{}{HEADER_MESSAGE_BACK}{}",
                        &payments.len(),
                        display_payments_paged(&payments, 0, &chat_id)
                    ),
                )
                .reply_markup(get_navigation_menu())
                .await?;

                // Logging
                log::info!(
                    "View Payments - User {} viewed payments for group {}, found {} payments",
                    sender_id,
                    chat_id,
                    &payments.len()
                );

                dialogue
                    .update(State::ViewPayments { payments, page: 0 })
                    .await?;
            }
            Err(ProcessError::CrudError(CrudError::NoPaymentsError())) => {
                send_bot_message(&bot, &msg, format!("😖 I can't find any payment records! But I'm always ready to help you get started with {COMMAND_ADD_PAYMENT}!"))
                    .await?;

                // Logging
                log::info!(
                    "View Payments - User {} viewed payments for group {}, but there were no payments recorded.",
                    sender_id,
                    chat_id,
                );

                dialogue.exit().await?;
            }
            Err(err) => {
                send_bot_message(&bot, &msg, format!("{UNKNOWN_ERROR_MESSAGE}")).await?;

                // Logging
                log::error!(
                    "View Payments - User {} failed to view payments for group {}: {}",
                    sender_id,
                    chat_id,
                    err.to_string()
                );

                dialogue.exit().await?;
            }
        }
    }
    Ok(())
}

/* Navigation function for user to interact with payment pagination menu.
*/
pub async fn action_view_more(
    bot: Bot,
    dialogue: UserDialogue,
    (payments, page): (Vec<Payment>, usize),
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(Message { id, chat, .. }) = query.message {
            let chat_id = chat.id.to_string();
            match button.as_str() {
                "Newer" => {
                    if page > 0 {
                        bot.edit_message_text(
                            chat_id.clone(),
                            id,
                            format!(
                                "{HEADER_MESSAGE_FRONT}{}{HEADER_MESSAGE_BACK}{}",
                                &payments.len(),
                                display_payments_paged(&payments, page - 1, &chat_id)
                            ),
                        )
                        .reply_markup(get_navigation_menu())
                        .await?;
                        dialogue
                            .update(State::ViewPayments {
                                payments,
                                page: page - 1,
                            })
                            .await?;
                    }
                }
                "Older" => {
                    if (page + 1) * 5 < payments.len() {
                        bot.edit_message_text(
                            chat_id.clone(),
                            id,
                            format!(
                                "{HEADER_MESSAGE_FRONT}{}{HEADER_MESSAGE_BACK}{}",
                                &payments.len(),
                                display_payments_paged(&payments, page + 1, &chat_id)
                            ),
                        )
                        .reply_markup(get_navigation_menu())
                        .await?;
                        dialogue
                            .update(State::ViewPayments {
                                payments,
                                page: page + 1,
                            })
                            .await?;
                    }
                }
                _ => {
                    log::error!(
                        "View Payments Menu - Invalid button in chat {}: {}",
                        chat.id,
                        button
                    );
                }
            }
        }
    }

    Ok(())
}

/* Entry point for edit payment function.
 * Bot responds by providing button menu of payments to choose from.
 * Points to SelectPayment state.
 */
pub async fn action_select_payment_edit(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (payments, page): (Vec<Payment>, usize),
) -> HandlerResult {
    let keyboard = get_select_menu(page, &payments);

    send_bot_message(
        &bot,
        &msg,
        "✏️ Which payment no. would you like to edit?".to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    dialogue
        .update(State::SelectPayment {
            payments,
            page,
            function: SelectPaymentType::EditPayment,
        })
        .await?;

    Ok(())
}

/* Entry point for delete payment function.
 * Bot responds by providing button menu of payments to choose from.
 * Points to SelectPayment state.
 */
pub async fn action_select_payment_delete(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    (payments, page): (Vec<Payment>, usize),
) -> HandlerResult {
    let keyboard = get_select_menu(page, &payments);

    send_bot_message(
        &bot,
        &msg,
        "🗑 Which payment no. would you like to delete?".to_string(),
    )
    .reply_markup(keyboard)
    .await?;

    dialogue
        .update(State::SelectPayment {
            payments,
            page,
            function: SelectPaymentType::DeletePayment,
        })
        .await?;

    Ok(())
}

/* Handles user response for selecting a payment.
 * Bot retrieves a callback query, and displays the payment.
 */
pub async fn action_select_payment_number(
    bot: Bot,
    dialogue: UserDialogue,
    query: CallbackQuery,
    state: State,
    (payments, page, function): (Vec<Payment>, usize, SelectPaymentType),
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(msg) = &query.message {
            let chat_id = msg.chat.id.to_string();
            let id = msg.id;
            match button.as_str() {
                "Cancel" => {
                    cancel_select_payment(bot, dialogue, query.message.unwrap(), state).await?;
                }
                num => {
                    let parsing = num.parse::<usize>();
                    if let Ok(serial_num) = parsing {
                        if serial_num <= payments.len() && serial_num > 0 {
                            let index = serial_num - 1;

                            match function {
                                SelectPaymentType::EditPayment => {
                                    action_edit_payment(
                                        bot,
                                        dialogue,
                                        msg,
                                        id,
                                        (payments, page),
                                        index,
                                    )
                                    .await?;
                                }
                                SelectPaymentType::DeletePayment => {
                                    action_delete_payment(
                                        bot,
                                        dialogue,
                                        msg,
                                        id,
                                        (payments, page),
                                        index,
                                    )
                                    .await?;
                                }
                            }
                        } else {
                            dialogue
                                .update(State::ViewPayments { payments, page })
                                .await?;

                            // Logging
                            log::error!(
                                "Select Payment Number - Invalid serial number {} in chat {}",
                                serial_num,
                                chat_id,
                            );
                        }
                    } else {
                        dialogue
                            .update(State::ViewPayments { payments, page })
                            .await?;

                        // Logging
                        log::error!(
                            "Select Payment Number - Invalid serial number {} in chat {}",
                            num,
                            chat_id,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
