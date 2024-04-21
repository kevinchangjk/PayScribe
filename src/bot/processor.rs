use std::ops::Neg;

use super::{
    currency::convert_currency,
    optimizer::optimize_debts,
    redis::{
        add_payment_entry, delete_payment_entry, get_chat_balances, get_chat_payments_details,
        get_currency_conversion, get_default_currency, get_payment_entry, get_time_zone,
        retrieve_chat_debts, set_currency_conversion, set_default_currency, set_time_zone,
        update_chat, update_chat_balances, update_chat_debts, update_payment_entry, update_user,
        CrudError, Debt, Payment, UserBalance, UserPayment, CURRENCY_CODE_DEFAULT,
    },
};

/* Processor is the overall logic center of the bot.
 * It handles the main logic, communicating with the front-facing handler
 * and the back-facing redis manager.
 * It defines and executes the main functions required of the bot,
 * and handles exceptions and errors in the back.
 */

#[derive(Debug, Clone)]
pub enum ChatSetting {
    TimeZone(Option<String>),
    DefaultCurrency(Option<String>),
    CurrencyConversion(Option<bool>),
}

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

// Sets "NIL" to default currency (which can be "NIL").
fn process_balances_currencies(
    mut balances: Vec<Vec<UserBalance>>,
    chat_id: &str,
) -> Vec<Vec<UserBalance>> {
    let default_currency = get_default_currency(chat_id);
    let default_currency = match default_currency {
        Ok(currency) => currency,
        Err(_) => CURRENCY_CODE_DEFAULT.to_string(),
    };

    let mut splits: Vec<Vec<UserBalance>> = vec![Vec::new()];
    let default_currency_index = 0;
    let mut curr_index = 0;
    for currency_balances in &mut balances {
        let target_index: usize;

        if currency_balances.len() > 0 {
            let currency = currency_balances[0].currency.as_str();
            if currency == CURRENCY_CODE_DEFAULT {
                currency_balances[0].currency = default_currency.clone();
                target_index = default_currency_index;
            } else if currency == default_currency {
                target_index = default_currency_index;
            } else {
                splits.push(Vec::new());
                curr_index += 1;
                target_index = curr_index;
            }

            for balance in &mut *currency_balances {
                let user_index = splits[target_index]
                    .iter()
                    .position(|bal| bal.username == balance.username);
                match user_index {
                    Some(user_index) => {
                        splits[target_index][user_index].balance += balance.balance;
                    }
                    None => {
                        splits[target_index].push(balance.clone());
                    }
                }
            }
        }
    }

    splits
}

// Converts all balances to default currency. Cannot be called when default is NIL.
async fn convert_balances_currencies(
    mut balances: Vec<Vec<UserBalance>>,
    chat_id: &str,
) -> Vec<UserBalance> {
    let default_currency = get_default_currency(chat_id);
    let default_currency = match default_currency {
        Ok(currency) => currency,
        Err(_) => CURRENCY_CODE_DEFAULT.to_string(),
    };
    let mut result: Vec<UserBalance> = Vec::new();

    for currency_balances in &mut balances {
        for balance in currency_balances {
            let currency = balance.currency.as_str();
            if currency == CURRENCY_CODE_DEFAULT {
                balance.currency = default_currency.clone();
            } else if currency != default_currency {
                balance.balance =
                    convert_currency(balance.balance, &balance.currency, &default_currency).await;
                balance.currency = default_currency.clone();
            }
            let index = result
                .iter()
                .position(|bal| bal.username == balance.username);
            match index {
                Some(index) => {
                    result[index].balance += balance.balance;
                }
                None => {
                    result.push(balance.clone());
                }
            }
        }
    }

    result
}

async fn update_balances_debts(
    chat_id: &str,
    changes: Vec<UserBalance>,
) -> Result<Vec<Debt>, ProcessError> {
    // Update balances
    update_chat_balances(chat_id, changes)?;
    let balances = get_chat_balances(chat_id)?;

    if balances.is_empty() {
        return Ok(Vec::new());
    }

    // Update group debts
    let mut all_debts = Vec::new();
    let conversion = get_currency_conversion(chat_id)?;
    if conversion {
        let balances = convert_balances_currencies(balances, chat_id).await;
        let debts = optimize_debts(balances);
        all_debts.extend(debts);
    } else {
        let processed_balances = process_balances_currencies(balances, chat_id);
        for split in processed_balances {
            let debts = optimize_debts(split);
            all_debts.extend(debts);
        }
    }

    update_chat_debts(&chat_id, &all_debts)?;

    Ok(all_debts)
}

/* Add a new payment entry in a group chat.
 * Execution flow: Updates relevant users, updates chat.
 * Adds payment entry, updates balances, updates group debts.
 * Important: assumes that debts sum up to total. Creditor's share included.
 */
pub async fn add_payment(
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

    update_balances_debts(&chat_id, changes).await
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
pub async fn edit_payment(
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

        let res = update_balances_debts(&chat_id, changes).await?;
        return Ok(Some(res));
    }

    Ok(None)
}

/* Delete a payment entry in a group chat.
 * Execution flow: Delete payment entry.
 * Update balances, update group debts.
 * Has to be called after self::view_payments.
 */
pub async fn delete_payment(chat_id: &str, payment_id: &str) -> Result<Vec<Debt>, ProcessError> {
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

    update_balances_debts(&chat_id, changes).await
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

/* Retrieves a group chat setting.
 */
pub fn get_chat_setting(chat_id: &str, setting: ChatSetting) -> Result<ChatSetting, ProcessError> {
    match setting {
        ChatSetting::TimeZone(_) => {
            let time_zone = get_time_zone(chat_id)?;
            Ok(ChatSetting::TimeZone(Some(time_zone)))
        }
        ChatSetting::DefaultCurrency(_) => {
            let currency = get_default_currency(chat_id)?;
            Ok(ChatSetting::DefaultCurrency(Some(currency)))
        }
        ChatSetting::CurrencyConversion(_) => {
            let convert = get_currency_conversion(chat_id)?;
            Ok(ChatSetting::CurrencyConversion(Some(convert)))
        }
    }
}

/* Sets a group chat setting.
 */
pub async fn set_chat_setting(chat_id: &str, setting: ChatSetting) -> Result<(), ProcessError> {
    match setting {
        ChatSetting::TimeZone(time_zone) => {
            if let Some(time_zone) = time_zone {
                set_time_zone(chat_id, &time_zone)?;
            }
        }
        ChatSetting::DefaultCurrency(currency) => {
            if let Some(currency) = currency {
                set_default_currency(chat_id, &currency)?;
            }
        }
        ChatSetting::CurrencyConversion(convert) => {
            if let Some(convert) = convert {
                set_currency_conversion(chat_id, convert)?;

                // If currency conversion is updated, need to update balances
                update_balances_debts(chat_id, Vec::new()).await?;
            }
        }
    }
    Ok(())
}

/* Changes the default currency of a group chat.
 * Also handles all the conversion logic for past payments.
 */
pub async fn update_chat_default_currency(
    chat_id: &str,
    currency: &str,
) -> Result<(), ProcessError> {
    let old_currency = get_default_currency(chat_id)?;

    // Update all payments to old currency
    let payments = get_chat_payments_details(chat_id);
    let mut changes: Vec<UserBalance> = Vec::new();

    match payments {
        Ok(payments) => {
            for payment in payments {
                if payment.payment.currency == CURRENCY_CODE_DEFAULT {
                    update_payment_entry(
                        &payment.payment_id,
                        None,
                        None,
                        Some(&old_currency),
                        None,
                        None,
                    )?;
                }
            }

            // Update all balances to old currency
            let balances = get_chat_balances(chat_id)?;
            for currency_balances in balances {
                if currency_balances.len() > 0
                    && currency_balances[0].currency == CURRENCY_CODE_DEFAULT
                {
                    for balance in currency_balances {
                        let change_sub = UserBalance {
                            username: balance.username.clone(),
                            currency: balance.currency,
                            balance: balance.balance.neg(),
                        };
                        let change_add = UserBalance {
                            username: balance.username.clone(),
                            currency: old_currency.clone(),
                            balance: balance.balance,
                        };
                        changes.extend(vec![change_sub, change_add]);
                    }
                }
            }
        }
        Err(_) => {
            // This means that there were no payments found
        }
    }

    // Update default currency in settings. If now NIL, disable currency conversion.
    set_default_currency(chat_id, &currency)?;
    if currency == CURRENCY_CODE_DEFAULT {
        set_currency_conversion(chat_id, false)?;
    }

    // Finally, update balances and debts
    update_balances_debts(chat_id, changes).await?;

    Ok(())
}
