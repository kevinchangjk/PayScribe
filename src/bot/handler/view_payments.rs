use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardMarkup, Message},
};

use crate::bot::{
    dispatcher::State,
    handler::utils::{
        display_payment, make_keyboard, HandlerResult, UserDialogue, COMMAND_ADD_PAYMENT,
        UNKNOWN_ERROR_MESSAGE,
    },
    processor::{view_payments, ProcessError},
    redis::{CrudError, UserPayment},
};

/* Utilities */
const HEADER_MESSAGE_FRONT: &str = "I've recorded ";
const HEADER_MESSAGE_BACK: &str = " payments for this group.\nHere are the latest ones:\n\n";

#[derive(Clone, Debug)]
pub struct Payment {
    pub payment_id: String,
    pub chat_id: String,
    pub datetime: String,
    pub description: String,
    pub creditor: String,
    pub total: f64,
    pub debts: Vec<(String, f64)>,
}

fn unfold_payment(payment: UserPayment) -> Payment {
    Payment {
        payment_id: payment.payment_id,
        chat_id: payment.chat_id,
        datetime: payment.payment.datetime,
        description: payment.payment.description,
        creditor: payment.payment.creditor,
        total: payment.payment.total,
        debts: payment.payment.debts,
    }
}

fn display_payments_paged(payments: &Vec<Payment>, page: usize) -> String {
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
        .map(|(index, payment)| display_payment(payment, serial_num + index));

    format!("{}", formatted_payments.collect::<Vec<String>>().join(""))
}

fn get_navigation_menu() -> InlineKeyboardMarkup {
    let buttons = vec!["Newer", "Older"];
    make_keyboard(buttons, Some(2))
}

/* View all payments.
 * Bot retrieves all payments, and displays the most recent 5.
 * Then, presents a previous and next page button for the user to navigate the pagination.
 */
pub async fn action_view_payments(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
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
                log::info!(
                    "View Payments - User {} viewed payments for group {}, found: {}",
                    sender_id,
                    chat_id,
                    display_payments_paged(&payments, 0)
                );
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "{HEADER_MESSAGE_FRONT}{}{HEADER_MESSAGE_BACK}{}",
                        &payments.len(),
                        display_payments_paged(&payments, 0)
                    ),
                )
                .reply_markup(get_navigation_menu())
                .await?;
                dialogue
                    .update(State::ViewPayments { payments, page: 0 })
                    .await?;
                return Ok(());
            }
            Err(ProcessError::CrudError(CrudError::NoPaymentsError())) => {
                bot.send_message(msg.chat.id, format!("I have not recorded any payment entry for this group. Use {COMMAND_ADD_PAYMENT} to let me know when you need my help with that!"))
                    .await?;
                dialogue.exit().await?;
            }
            Err(err) => {
                log::error!(
                    "View Payments - User {} failed to view payments for group {}: {}",
                    sender_id,
                    chat_id,
                    err.to_string()
                );
                bot.send_message(msg.chat.id, format!("{UNKNOWN_ERROR_MESSAGE}"))
                    .await?;
            }
        }
    }
    dialogue.exit().await?;
    log::error!(
        "View Payments - User not found in msg {}.",
        msg.id.to_string()
    );
    Ok(())
}

pub async fn action_view_more(
    bot: Bot,
    dialogue: UserDialogue,
    (payments, page): (Vec<Payment>, usize),
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(query.id.to_string()).await?;

        if let Some(Message { id, chat, .. }) = query.message {
            match button.as_str() {
                "Newer" => {
                    if page > 0 {
                        bot.edit_message_text(
                            chat.id,
                            id,
                            format!(
                                "{HEADER_MESSAGE_FRONT}{}{HEADER_MESSAGE_BACK}{}",
                                &payments.len(),
                                display_payments_paged(&payments, page - 1)
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
                            chat.id,
                            id,
                            format!(
                                "{HEADER_MESSAGE_FRONT}{}{HEADER_MESSAGE_BACK}{}",
                                &payments.len(),
                                display_payments_paged(&payments, page + 1)
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
