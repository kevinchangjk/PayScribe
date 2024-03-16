use std::ops::Neg;

use chrono::Duration;
use teloxide::{prelude::*, types::ChatPermissions};

use super::{
    optimizer::optimize_debts,
    redis::{
        add_payment_entry, delete_payment_entry, get_chat_payments_details, retrieve_chat_debts,
        test_redis_connection, update_chat, update_chat_balances, update_chat_debts,
        update_payment_entry, update_user, CrudError, Debt, Payment, UserBalance, UserPayment,
    },
};

/* Processor is the overall logic center of the bot.
 * It handles the main logic, communicating with the front-facing handler
 * and the back-facing redis manager.
 * It defines and executes the main functions required of the bot,
 * and handles exceptions and errors in the back.
 */

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ProcessError {
    #[error("Database CRUD error: {0}")]
    CrudError(CrudError),
}

// Implement the From trait to convert from CrudError to ProcessError
impl From<CrudError> for ProcessError {
    fn from(crud_error: CrudError) -> ProcessError {
        ProcessError::CrudError(crud_error)
    }
}

/* Utility functions */
fn auto_update_user(msg: Message) -> Result<(), ProcessError> {
    let chat_id = msg.chat.id.to_string();
    if let Some(user) = msg.from() {
        if let Some(username) = &user.username {
            update_user(username, &chat_id, Some(&user.id.to_string()))?;
        }
    }
    Ok(())
}

fn update_balances_debts(
    chat_id: &str,
    changes: Vec<UserBalance>,
) -> Result<Vec<Debt>, ProcessError> {
    // Update balances
    let balances = update_chat_balances(chat_id, changes)?;

    // Update group debts
    let debts = optimize_debts(balances);
    update_chat_debts(&chat_id, debts.clone())?;

    Ok(debts)
}

/* Add a new payment entry in a group chat.
 * Execution flow: Updates relevant users, updates chat.
 * Adds payment entry, updates balances, updates group debts.
 */
pub fn add_payment(
    msg: Message,
    description: &str,
    creditor: &str,
    total: f64,
    debts: Vec<(String, f64)>,
) -> Result<Vec<Debt>, ProcessError> {
    let chat_id = msg.chat.id.to_string();
    let mut all_users = vec![creditor.to_string()];

    for (user, _) in debts.iter() {
        all_users.push(user.to_string());
    }

    // Update all users included in payment
    for user in all_users.iter() {
        update_user(user, &chat_id, None)?;
    }

    // Add message sender to the list of users
    if let Some(user) = msg.from() {
        if let Some(username) = &user.username {
            update_user(username, &chat_id, Some(&user.id.to_string()))?;
            all_users.push(username.to_string());
        }
    }

    // Update chat
    update_chat(&chat_id, all_users)?;

    // Add payment entry
    let payment = Payment {
        description: description.to_string(),
        datetime: msg.date.to_string(),
        creditor: creditor.to_string(),
        total,
        debts: debts.clone(),
    };
    add_payment_entry(&chat_id, &payment)?;

    // Update balances
    let mut changes: Vec<UserBalance> = debts
        .iter()
        .map(|(user, amount)| UserBalance {
            username: user.to_string(),
            balance: amount.neg(),
        })
        .collect();
    changes.push(UserBalance {
        username: creditor.to_string(),
        balance: total,
    });

    update_balances_debts(&chat_id, changes)
}

/* View all payment entries of a group chat.
 * Execution flow: Retrieve chat payment details.
 * Called only once per command. Pagination handled by Handler.
 */
pub fn view_payments(msg: Message) -> Result<Vec<UserPayment>, ProcessError> {
    auto_update_user(msg.clone())?;

    let chat_id = msg.chat.id.to_string();
    let payments = get_chat_payments_details(&chat_id)?;
    Ok(payments)
}

/* Edit a payment entry in a group chat.
 * Execution flow: Edit payment entry.
 * Update balances, update group debts.
 * Has to be called after self::view_payments.
 */
pub fn edit_payment(
    msg: Message,
    payment_id: &str,
    current_payment: Payment,
    description: Option<&str>,
    creditor: Option<&str>,
    total: Option<&f64>,
    debts: Option<Vec<(String, f64)>>,
) -> Result<Option<Vec<Debt>>, ProcessError> {
    // Edit payment entry
    update_payment_entry(payment_id, description, creditor, total, debts.clone())?;

    // Update balances in two stages: first undo the previous payment, then set the new one
    if let Some(creditor) = creditor {
        if let Some(total) = total {
            if let Some(debts) = debts {
                let chat_id = msg.chat.id.to_string();

                // First round of update
                let mut prev_changes: Vec<UserBalance> = current_payment
                    .debts
                    .iter()
                    .map(|debt| UserBalance {
                        username: debt.0.to_string(),
                        balance: debt.1,
                    })
                    .collect();
                prev_changes.push(UserBalance {
                    username: current_payment.creditor,
                    balance: current_payment.total.neg(),
                });
                update_chat_balances(&chat_id, prev_changes)?;

                // Second round of update
                let mut changes: Vec<UserBalance> = debts
                    .iter()
                    .map(|debt| UserBalance {
                        username: debt.0.to_string(),
                        balance: debt.1.neg(),
                    })
                    .collect();
                changes.push(UserBalance {
                    username: creditor.to_string(),
                    balance: *total,
                });

                let res = update_balances_debts(&chat_id, changes)?;
                return Ok(Some(res));
            }
        }
    }

    Ok(None)
}

/* Delete a payment entry in a group chat.
 * Execution flow: Delete payment entry.
 * Update balances, update group debts.
 * Has to be called after self::view_payments.
 */
pub fn delete_payment(
    msg: Message,
    payment_id: &str,
    current_payment: Payment,
) -> Result<Vec<Debt>, ProcessError> {
    // Delete payment entry
    let chat_id = msg.chat.id.to_string();
    delete_payment_entry(&chat_id, payment_id)?;

    // Update balances
    let mut changes: Vec<UserBalance> = current_payment
        .debts
        .iter()
        .map(|debt| UserBalance {
            username: debt.0.to_string(),
            balance: debt.1,
        })
        .collect();
    changes.push(UserBalance {
        username: current_payment.creditor,
        balance: current_payment.total.neg(),
    });

    update_balances_debts(&chat_id, changes)
}

/* View all debts (balances) of a group chat.
 * Execution flow: Retrieve all debts.
 */
pub fn view_debts(msg: Message) -> Result<Vec<Debt>, ProcessError> {
    auto_update_user(msg.clone())?;

    let chat_id = msg.chat.id.to_string();
    let debts = retrieve_chat_debts(&chat_id)?;
    Ok(debts)
}
