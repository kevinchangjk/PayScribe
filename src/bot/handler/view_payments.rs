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
            process_debts,
        },
    },
    processor::{add_payment, view_payments},
    redis::UserPayment,
    BotError,
};

/* Utilities */
const HEADER_MESSAGE: &str = "Adding a new payment entry!\n\n";
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

fn display_payment(payment: &Payment) -> String {
    format!(
        "Date: {}\nDescription: {}\nCreditor: {}\nTotal: {}\nSplit amounts: {}\n",
        payment.datetime,
        payment.description,
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
    format!(
        "{}",
        displayed_payments
            .iter()
            .map(|payment| display_payment(payment))
            .collect::<Vec<String>>()
            .join("\n")
    )
}

fn get_navigation_menu() -> InlineKeyboardMarkup {
    let buttons = vec!["Newer, Older"];
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
                            "Here are the payments for this group!\n\n{}",
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

pub async fn action_view_more(bot: Bot, dialogue: UserDialogue, msg: Message) -> HandlerResult {
    Ok(())
}
