use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardMarkup, Message},
};

use crate::bot::{
    dispatcher::{HandlerResult, State, UserDialogue},
    handler::{
        general::{NO_TEXT_MESSAGE, UNKNOWN_ERROR_MESSAGE},
        utils::{
            display_balances, display_debts, make_keyboard, parse_amount, parse_username,
            process_debts, reformat_datetime,
        },
    },
    processor::{add_payment, view_payments},
    redis::UserPayment,
    BotError,
};

/* Utilities */
const HEADER_MESSAGE: &str = " payments tracked for this group!\n\n";
const FOOTER_MESSAGE: &str = "\n\n";

#[derive(Clone, Debug)]
pub struct Payment {
    payment_id: String,
    chat_id: String,
    datetime: String,
    description: String,
    creditor: String,
    total: f64,
    debts: Vec<(String, f64)>,
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

fn display_payment(payment: &Payment, serialNum: usize) -> String {
    format!(
        "______________________________\nEntry No. {} — {}\nDate: {}\nCreditor: {}\nTotal: {:.2}\nSplit amounts:\n{}",
        serialNum,
        payment.description,
        reformat_datetime(&payment.datetime),
        payment.creditor,
        payment.total,
        display_debts(&payment.debts)
    )
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

    format!("{}", formatted_payments.collect::<Vec<String>>().join("\n"))
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
                if payments.is_empty() {
                    log::info!(
                        "View Payments - User {} viewed payments for group {}, but no payments found.",
                        sender_id,
                        chat_id
                    );
                    bot.send_message(msg.chat.id, format!("No payments found for this group!"))
                        .await?;
                    dialogue.exit().await?;
                } else {
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
                            "{}{HEADER_MESSAGE}{}",
                            &payments.len(),
                            display_payments_paged(&payments, 0)
                        ),
                    )
                    .reply_markup(get_navigation_menu())
                    .await?;
                    dialogue
                        .update(State::ViewPayments { payments, page: 0 })
                        .await?;
                }
                return Ok(());
            }
            Err(err) => {
                dialogue.exit().await?;
                return Err(BotError::UserError(format!(
                    "View Payments - User {} failed to view payments for group {}: {}",
                    sender_id,
                    chat_id,
                    err.to_string()
                )));
            }
        }
    }
    dialogue.exit().await?;
    Err(BotError::UserError(
        "Unable to view payments: User not found".to_string(),
    ))
}

pub async fn action_view_more(
    bot: Bot,
    dialogue: UserDialogue,
    (payments, page): (Vec<Payment>, usize),
    query: CallbackQuery,
) -> HandlerResult {
    if let Some(button) = &query.data {
        bot.answer_callback_query(format!("{}", query.id)).await?;

        if let Some(Message { id, chat, .. }) = query.message {
            match button.as_str() {
                "Newer" => {
                    if page > 0 {
                        bot.edit_message_text(
                            chat.id,
                            id,
                            format!(
                                "{}{HEADER_MESSAGE}{}",
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
                                "{}{HEADER_MESSAGE}{}",
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
