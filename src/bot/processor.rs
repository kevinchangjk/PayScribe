use std::ops::Neg;

use super::{
    optimizer::optimize_debts,
    redis::{
        add_payment_entry, delete_payment_entry, get_chat_payments_details, get_payment_entry,
        retrieve_chat_debts, update_chat, update_chat_balances, update_chat_debts,
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
    #[error("{0}")]
    CrudError(CrudError),
}

// Implement the From trait to convert from CrudError to ProcessError
impl From<CrudError> for ProcessError {
    fn from(crud_error: CrudError) -> ProcessError {
        ProcessError::CrudError(crud_error)
    }
}

/* Utility functions */
fn auto_update_user(
    chat_id: &str,
    sender_id: &str,
    sender_username: Option<&str>,
) -> Result<(), ProcessError> {
    if let Some(username) = sender_username {
        update_user(&username, chat_id, Some(sender_id))?;
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
    update_chat_debts(&chat_id, &debts)?;

    Ok(debts)
}

/* Add a new payment entry in a group chat.
 * Execution flow: Updates relevant users, updates chat.
 * Adds payment entry, updates balances, updates group debts.
 * Important: assumes that debts sum up to total. Creditor's share included.
 */
pub fn add_payment(
    chat_id: String,
    sender_username: String,
    sender_id: String,
    datetime: String,
    description: &str,
    creditor: &str,
    currency: &str,
    total: i64,
    debts: Vec<(String, i64)>,
) -> Result<Vec<Debt>, ProcessError> {
    let mut all_users = vec![creditor.to_string()];

    for (user, _) in debts.iter() {
        if user == &creditor {
            continue;
        }
        all_users.push(user.to_string());
    }

    // Update all users included in payment
    let mut is_sender_included = false;
    for user in all_users.iter() {
        if user == &sender_username {
            is_sender_included = true;
            continue;
        }
        update_user(user, &chat_id, None)?;
    }

    // Add message sender to the list of users
    update_user(&sender_username, &chat_id, Some(&sender_id))?;
    if !is_sender_included {
        all_users.push(sender_username);
    }

    // Update chat
    update_chat(&chat_id, all_users)?;

    // Add payment entry
    let payment = Payment {
        description: description.to_string(),
        datetime,
        creditor: creditor.to_string(),
        currency: currency.to_string(),
        total,
        debts: debts.clone(),
    };
    add_payment_entry(&chat_id, &payment)?;

    // Update balances
    let mut changes: Vec<UserBalance> = debts
        .iter()
        .map(|(user, amount)| UserBalance {
            username: user.to_string(),
            currency: currency.to_string(),
            balance: amount.neg(),
        })
        .collect();

    changes.push(UserBalance {
        username: creditor.to_string(),
        currency: currency.to_string(),
        balance: total,
    });

    update_balances_debts(&chat_id, changes)
}

/* View all payment entries of a group chat.
 * Execution flow: Retrieve chat payment details.
 * Called only once per command. Pagination handled by Handler.
 */
pub fn view_payments(
    chat_id: &str,
    sender_id: &str,
    sender_username: Option<&str>,
) -> Result<Vec<UserPayment>, ProcessError> {
    auto_update_user(chat_id, sender_id, sender_username)?;

    let payments = get_chat_payments_details(&chat_id)?;
    Ok(payments)
}

/* Edit a payment entry in a group chat.
 * Execution flow: Edit payment entry.
 * Update balances, update group debts.
 * Has to be called after self::view_payments.
 */
pub fn edit_payment(
    chat_id: &str,
    payment_id: &str,
    description: Option<&str>,
    creditor: Option<&str>,
    currency: Option<&str>,
    total: Option<&i64>,
    debts: Option<Vec<(String, i64)>>,
) -> Result<Option<Vec<Debt>>, ProcessError> {
    // Get current payment entry
    let current_payment = get_payment_entry(payment_id)?;

    // Edit payment entry
    update_payment_entry(
        payment_id,
        description,
        creditor,
        currency,
        total,
        debts.clone(),
    )?;

    // Update balances in two stages: first undo the previous payment, then set the new one
    if creditor.is_some() || total.is_some() || debts.is_some() {
        // First round of update
        let prev_creditor = &current_payment.creditor;
        let prev_currency = &current_payment.currency;
        let mut prev_changes: Vec<UserBalance> = current_payment
            .debts
            .iter()
            .map(|debt| UserBalance {
                username: debt.0.to_string(),
                currency: prev_currency.to_string(),
                balance: debt.1,
            })
            .collect();
        prev_changes.push(UserBalance {
            username: prev_creditor.to_string(),
            currency: prev_currency.to_string(),
            balance: current_payment.total.neg(),
        });
        update_chat_balances(&chat_id, prev_changes.clone())?;

        // Second round of update
        let mut changes: Vec<UserBalance> = debts
            .unwrap_or(current_payment.debts)
            .iter()
            .map(|debt| UserBalance {
                username: debt.0.to_string(),
                currency: currency.unwrap_or(prev_currency).to_string(),
                balance: debt.1.neg(),
            })
            .collect();
        changes.push(UserBalance {
            username: creditor.unwrap_or(&current_payment.creditor).to_string(),
            currency: currency.unwrap_or(prev_currency).to_string(),
            balance: *total.unwrap_or(&current_payment.total),
        });

        let res = update_balances_debts(&chat_id, changes)?;
        return Ok(Some(res));
    }

    Ok(None)
}

/* Delete a payment entry in a group chat.
 * Execution flow: Delete payment entry.
 * Update balances, update group debts.
 * Has to be called after self::view_payments.
 */
pub fn delete_payment(chat_id: &str, payment_id: &str) -> Result<Vec<Debt>, ProcessError> {
    // Get payment entry
    let payment = get_payment_entry(payment_id)?;

    // Delete payment entry
    delete_payment_entry(&chat_id, payment_id)?;

    // Update balances
    let mut changes: Vec<UserBalance> = payment
        .debts
        .iter()
        .map(|debt| UserBalance {
            username: debt.0.to_string(),
            currency: payment.currency.clone(),
            balance: debt.1,
        })
        .collect();
    changes.push(UserBalance {
        username: payment.creditor,
        currency: payment.currency,
        balance: payment.total.neg(),
    });

    update_balances_debts(&chat_id, changes)
}

/* View all debts (balances) of a group chat.
 * Execution flow: Retrieve all debts.
 */
pub fn view_debts(
    chat_id: &str,
    sender_id: &str,
    sender_username: Option<&str>,
) -> Result<Vec<Debt>, ProcessError> {
    auto_update_user(chat_id, sender_id, sender_username)?;

    let debts = retrieve_chat_debts(&chat_id)?;
    Ok(debts)
}
