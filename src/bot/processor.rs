use std::ops::Neg;

use super::{
    currency::{convert_currency_with_rate, fetch_currency_conversion},
    handler::StatementOption,
    optimizer::optimize_debts,
    redis::{
        add_payment_entry, delete_payment_entry, get_chat_balances, get_chat_balances_currency,
        get_chat_payments_details, get_currency_conversion, get_default_currency,
        get_erase_messages, get_payment_entry, get_time_zone, get_valid_chat_currencies,
        is_request_limit_exceeded, retrieve_chat_spendings, retrieve_chat_spendings_currency,
        set_currency_conversion, set_default_currency, set_erase_messages, set_time_zone,
        update_chat, update_chat_balances, update_chat_spendings, update_payment_entry,
        update_user, CrudError, Debt, Payment, UserBalance, UserPayment, CURRENCY_CODE_DEFAULT,
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
    DefaultCurrency(Option<String>),
    CurrencyConversion(Option<bool>),
    EraseMessages(Option<bool>),
    TimeZone(Option<String>),
}

#[derive(Debug, Clone)]
pub struct UserSpending {
    pub username: String,
    pub spending: i64,
    pub paid: i64,
}

#[derive(Debug, Clone)]
pub struct SpendingData {
    pub currency: String,
    pub group_spending: i64,
    pub user_spendings: Vec<UserSpending>,
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
pub fn is_username_equal(first: &str, second: &str) -> bool {
    first.to_lowercase() == second.to_lowercase()
}

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

fn update_balances(chat_id: &str, changes: Vec<UserBalance>) -> Result<(), ProcessError> {
    // Update balances
    update_chat_balances(chat_id, changes)?;
    Ok(())
}

// Utility function required by many commands.
// Updates the balances based on changes that are given.
// Returns the debts for the relevant currency.
async fn update_balances_debts(
    chat_id: &str,
    changes: Vec<UserBalance>,
    currency: StatementOption,
) -> Result<Vec<Debt>, ProcessError> {
    update_balances(chat_id, changes)?;

    let debts = retrieve_debts(chat_id, currency).await?;

    Ok(debts)
}

// Updates users and chat given payment details
fn update_users_chat(
    chat_id: &str,
    sender_username: &str,
    sender_id: &str,
    creditor: Option<&str>,
    debts: Option<Vec<(String, i64)>>,
) -> Result<(), ProcessError> {
    let mut all_users = vec![];

    if let Some(creditor) = creditor {
        all_users.push(creditor.to_string());
    }

    if let Some(debts) = debts {
        for (user, _) in debts.iter() {
            if is_username_equal(user, creditor.unwrap_or("")) {
                continue;
            }
            all_users.push(user.to_string());
        }
    }

    // Update all users included in payment
    let mut is_sender_included = false;
    for user in all_users.iter() {
        if is_username_equal(user, sender_username) {
            is_sender_included = true;
            continue;
        }
        update_user(user, chat_id, None)?;
    }

    // Add message sender to the list of users
    update_user(sender_username, chat_id, Some(sender_id))?;
    if !is_sender_included {
        all_users.push(sender_username.to_string());
    }

    // Update chat
    update_chat(&chat_id, all_users)?;

    Ok(())
}

pub fn init_chat_config(chat_id: &str) -> Result<(), ProcessError> {
    update_chat(chat_id, Vec::new())?;
    Ok(())
}

/* Retrieves all valid currencies for a chat.
 * Valid currencies are currencies with some payments.
 */
pub fn retrieve_valid_currencies(chat_id: &str) -> Result<Vec<String>, ProcessError> {
    let currencies = get_valid_chat_currencies(chat_id)?;
    Ok(currencies)
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
    // Update users and chat
    update_users_chat(
        &chat_id,
        &sender_username,
        &sender_id,
        Some(creditor),
        Some(debts.clone()),
    )?;

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

    // Update spendings
    let spendings: Vec<UserBalance> = debts
        .iter()
        .map(|(user, amount)| UserBalance {
            username: user.to_string(),
            currency: currency.to_string(),
            balance: *amount,
        })
        .collect();
    update_chat_spendings(&chat_id, spendings)?;

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

    let conversion = get_currency_conversion(&chat_id)?;
    let option = if conversion {
        StatementOption::ConvertCurrency
    } else {
        StatementOption::Currency(currency.to_string())
    };

    update_balances_debts(&chat_id, changes, option).await
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
    sender_username: String,
    sender_id: String,
    payment_id: &str,
    description: Option<&str>,
    creditor: Option<&str>,
    currency: Option<&str>,
    total: Option<&i64>,
    debts: Option<Vec<(String, i64)>>,
) -> Result<Option<Vec<Debt>>, ProcessError> {
    // Get current payment entry
    let current_payment = get_payment_entry(payment_id)?;

    // Update users and chat
    update_users_chat(
        &chat_id,
        &sender_username,
        &sender_id,
        creditor,
        debts.clone(),
    )?;

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
        update_chat_balances(&chat_id, prev_changes)?;

        // Update spendings as well
        let prev_spendings: Vec<UserBalance> = current_payment
            .debts
            .iter()
            .map(|debt| UserBalance {
                username: debt.0.to_string(),
                currency: prev_currency.to_string(),
                balance: debt.1.neg(),
            })
            .collect();
        update_chat_spendings(&chat_id, prev_spendings)?;

        // Second round of update
        let mut changes: Vec<UserBalance> = debts
            .clone()
            .unwrap_or(current_payment.debts.clone())
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

        // Update spendings as well
        let new_spendings: Vec<UserBalance> = debts
            .unwrap_or(current_payment.debts)
            .iter()
            .map(|debt| UserBalance {
                username: debt.0.to_string(),
                currency: currency.unwrap_or(prev_currency).to_string(),
                balance: debt.1,
            })
            .collect();
        update_chat_spendings(&chat_id, new_spendings)?;

        let conversion = get_currency_conversion(&chat_id)?;
        let option = if conversion {
            StatementOption::ConvertCurrency
        } else {
            StatementOption::Currency(currency.unwrap_or(prev_currency).to_string())
        };

        let res = update_balances_debts(&chat_id, changes, option).await?;
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

    // Update spendings
    let spendings: Vec<UserBalance> = payment
        .debts
        .iter()
        .map(|debt| UserBalance {
            username: debt.0.to_string(),
            currency: payment.currency.clone(),
            balance: debt.1.neg(),
        })
        .collect();
    update_chat_spendings(&chat_id, spendings)?;

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
        currency: payment.currency.clone(),
        balance: payment.total.neg(),
    });

    let conversion = get_currency_conversion(&chat_id)?;
    let option = if conversion {
        StatementOption::ConvertCurrency
    } else {
        StatementOption::Currency(payment.currency.clone())
    };

    update_balances_debts(&chat_id, changes, option).await
}

/* View balances of a group chat.
 * Takes in a specification of the options for viewing.
 * Which is whether the currency is to be converted, and which currency.
 */
pub async fn retrieve_debts(
    chat_id: &str,
    option: StatementOption,
) -> Result<Vec<Debt>, ProcessError> {
    match option {
        StatementOption::Currency(currency) => retrieve_debts_by_currency(chat_id, &currency),
        StatementOption::ConvertCurrency => retrieve_debts_converted(chat_id).await,
    }
}

/* View debts of a group chat for a specific currency.
 * Retrieves all balances, optimizes debts, and returns.
 */
fn retrieve_debts_by_currency(chat_id: &str, currency: &str) -> Result<Vec<Debt>, ProcessError> {
    let default_currency = match get_chat_setting(chat_id, ChatSetting::DefaultCurrency(None))? {
        ChatSetting::DefaultCurrency(Some(curr)) => curr,
        _ => CURRENCY_CODE_DEFAULT.to_string(),
    };

    // If currency is NIL, which is not default currency.
    if currency == CURRENCY_CODE_DEFAULT && default_currency != CURRENCY_CODE_DEFAULT {
        return retrieve_debts_by_default_currency(chat_id);
    }

    // If currency is default currency, which is not NIL.
    if default_currency == currency && currency != CURRENCY_CODE_DEFAULT {
        return retrieve_debts_by_default_currency(chat_id);
    }

    // If currency is not NIL, and is not default currency.
    // Also, if currency is NIL, and NIL is default currency.
    let balances = get_chat_balances_currency(chat_id, currency)?;
    let debts = optimize_debts(balances);

    Ok(debts)
}

/* View debts of a group chat for the default currency.
 * Retrieves all balances, optimizes debts, and returns.
 */
fn retrieve_debts_by_default_currency(chat_id: &str) -> Result<Vec<Debt>, ProcessError> {
    let currency = match get_chat_setting(chat_id, ChatSetting::DefaultCurrency(None))? {
        ChatSetting::DefaultCurrency(Some(curr)) => curr,
        _ => CURRENCY_CODE_DEFAULT.to_string(),
    };
    let mut balances_curr = get_chat_balances_currency(chat_id, &currency)?;
    let balances_nil = get_chat_balances_currency(chat_id, CURRENCY_CODE_DEFAULT)?;

    for balance in balances_nil {
        let curr_index = balances_curr
            .iter()
            .position(|bal| bal.username == balance.username);
        match curr_index {
            Some(index) => {
                balances_curr[index].balance += balance.balance;
            }
            None => {
                balances_curr.push(balance);
            }
        }
    }

    let debts = optimize_debts(balances_curr);

    Ok(debts)
}

/* View debts of a group chat for all currencies, converted to default currency.
 * Retrieves all balances, optimizes debts, and returns.
 */
async fn retrieve_debts_converted(chat_id: &str) -> Result<Vec<Debt>, ProcessError> {
    let mut balances = get_chat_balances(chat_id)?;
    let default_currency = get_default_currency(chat_id)?;

    let mut converted_balances: Vec<UserBalance> = Vec::new();
    for balances_currency in &mut balances {
        if balances_currency.len() == 0 {
            continue;
        }

        let currency = balances_currency[0].currency.clone();
        let should_convert = currency != default_currency && currency != CURRENCY_CODE_DEFAULT;

        let conversion_rate = if should_convert {
            match fetch_currency_conversion(&currency, &default_currency).await {
                Ok(rate) => rate,
                Err(err) => {
                    log::error!("Error fetching currency conversion from {currency} to {default_currency}: {}", err);
                    1.0
                }
            }
        } else {
            1.0
        };

        for balance in balances_currency {
            let mut amount = balance.balance;
            if should_convert {
                amount = convert_currency_with_rate(
                    amount,
                    &currency,
                    &default_currency,
                    conversion_rate,
                );
            }
            let user = converted_balances
                .iter()
                .position(|bal| bal.username == balance.username);

            match user {
                Some(index) => {
                    converted_balances[index].balance += amount;
                }
                None => {
                    converted_balances.push(UserBalance {
                        username: balance.username.clone(),
                        currency: default_currency.clone(),
                        balance: amount,
                    });
                }
            }
        }
    }

    let debts = optimize_debts(converted_balances);

    Ok(debts)
}

/* View spendings of a group chat.
 * Takes in a specification of the options for viewing.
 * Which is whether the currency is to be converted, and which currency.
 */
pub async fn retrieve_spending_data(
    chat_id: &str,
    option: StatementOption,
) -> Result<SpendingData, ProcessError> {
    match option {
        StatementOption::Currency(currency) => {
            retrieve_spending_data_by_currency(chat_id, &currency)
        }
        StatementOption::ConvertCurrency => retrieve_spending_data_converted(chat_id).await,
    }
}

/* View spendings of a group chat for a specific currency.
 * Retrieves all spendings, gets current balances, and returns:
 * Total group spending, total individual spendings, and total individual payments
 */
fn retrieve_spending_data_by_currency(
    chat_id: &str,
    currency: &str,
) -> Result<SpendingData, ProcessError> {
    let default_currency = match get_chat_setting(chat_id, ChatSetting::DefaultCurrency(None))? {
        ChatSetting::DefaultCurrency(Some(curr)) => curr,
        _ => CURRENCY_CODE_DEFAULT.to_string(),
    };
    if default_currency == currency && currency != CURRENCY_CODE_DEFAULT {
        return retrieve_spending_data_by_default_currency(chat_id, currency);
    }

    let spendings = retrieve_chat_spendings_currency(chat_id, currency)?;
    let balances = get_chat_balances_currency(chat_id, currency)?;

    let mut group_spending = 0;
    let mut user_spendings: Vec<UserSpending> = Vec::new();
    for spending in spendings {
        group_spending += spending.balance;

        let balance = balances
            .iter()
            .find(|bal| bal.username == spending.username)
            .map(|bal| bal.balance)
            .unwrap_or(0);
        let paid = spending.balance + balance;

        if spending.balance != 0 || paid != 0 {
            user_spendings.push(UserSpending {
                username: spending.username.clone(),
                spending: spending.balance,
                paid,
            });
        }
    }

    // Check through for any balances that aren't accounted for in a spending
    for balance in balances {
        let user = user_spendings
            .iter()
            .position(|spending| spending.username == balance.username);
        match user {
            Some(_) => {
                continue;
            }
            None => {
                user_spendings.push(UserSpending {
                    username: balance.username,
                    spending: 0,
                    paid: balance.balance,
                });
            }
        }
    }

    Ok(SpendingData {
        currency: currency.to_string(),
        group_spending,
        user_spendings,
    })
}

/* View spendings of a group chat for the default currency.
 * Retrieves all spendings, gets current balances, and returns:
 * Total group spending, total individual spendings, and total individual payments
 */
fn retrieve_spending_data_by_default_currency(
    chat_id: &str,
    currency: &str,
) -> Result<SpendingData, ProcessError> {
    let spendings_curr = retrieve_chat_spendings_currency(chat_id, currency)?;
    let spendings_nil = retrieve_chat_spendings_currency(chat_id, CURRENCY_CODE_DEFAULT)?;
    let mut balances_curr = get_chat_balances_currency(chat_id, currency)?;
    let mut balances_nil = get_chat_balances_currency(chat_id, CURRENCY_CODE_DEFAULT)?;

    let mut group_spending = 0;
    let mut user_spendings: Vec<UserSpending> = Vec::new();
    for spending in spendings_curr {
        group_spending += spending.balance;

        let paid: i64;
        let balance_index = balances_curr
            .iter()
            .position(|bal| bal.username == spending.username);
        match balance_index {
            Some(index) => {
                paid = spending.balance + balances_curr[index].balance;
                balances_curr[index].balance = 0;
            }
            None => {
                paid = 0;
            }
        }

        user_spendings.push(UserSpending {
            username: spending.username.clone(),
            spending: spending.balance,
            paid,
        });
    }

    for spending in spendings_nil {
        group_spending += spending.balance;

        let paid: i64;
        let balance_index = balances_nil
            .iter()
            .position(|bal| bal.username == spending.username);
        match balance_index {
            Some(index) => {
                paid = spending.balance + balances_nil[index].balance;
                balances_nil[index].balance = 0;
            }
            None => {
                paid = 0;
            }
        }

        let user = user_spendings
            .iter()
            .position(|user| user.username == spending.username);

        match user {
            Some(index) => {
                user_spendings[index].spending += spending.balance;
                user_spendings[index].paid += paid;
            }
            None => {
                user_spendings.push(UserSpending {
                    username: spending.username.clone(),
                    spending: spending.balance,
                    paid,
                });
            }
        }
    }

    // Check through for any balances that aren't accounted for in a spending
    for balance in balances_curr {
        if balance.balance != 0 {
            let user = user_spendings
                .iter()
                .position(|spending| spending.username == balance.username);
            match user {
                Some(index) => {
                    user_spendings[index].paid += balance.balance;
                }
                None => {
                    user_spendings.push(UserSpending {
                        username: balance.username,
                        spending: 0,
                        paid: balance.balance,
                    });
                }
            }
        }
    }

    for balance in balances_nil {
        if balance.balance != 0 {
            let user = user_spendings
                .iter()
                .position(|spending| spending.username == balance.username);
            match user {
                Some(index) => {
                    user_spendings[index].paid += balance.balance;
                }
                None => {
                    user_spendings.push(UserSpending {
                        username: balance.username,
                        spending: 0,
                        paid: balance.balance,
                    });
                }
            }
        }
    }

    Ok(SpendingData {
        currency: currency.to_string(),
        group_spending,
        user_spendings,
    })
}

/* View spendings of a group chat, converted to default currency.
 * Only called if currency conversion is enabled.
 * Retrieves all spendings, gets current balances, converts them.
 */
async fn retrieve_spending_data_converted(chat_id: &str) -> Result<SpendingData, ProcessError> {
    let mut spendings = retrieve_chat_spendings(chat_id)?;
    let mut balances = get_chat_balances(chat_id)?;

    let default_currency = get_default_currency(chat_id)?;
    let mut group_spending = 0;
    let mut user_spendings: Vec<UserSpending> = Vec::new();
    for spending_currency in &mut spendings {
        if spending_currency.len() == 0 {
            continue;
        }

        let currency = spending_currency[0].currency.clone();
        let should_convert = currency != default_currency && currency != CURRENCY_CODE_DEFAULT;

        let conversion_rate = if should_convert {
            match fetch_currency_conversion(&currency, &default_currency).await {
                Ok(rate) => rate,
                Err(err) => {
                    log::error!("Error fetching currency conversion from {currency} to {default_currency}: {}", err);
                    1.0
                }
            }
        } else {
            1.0
        };

        let mut empty_balances: Vec<UserBalance> = Vec::new();
        let balances_currency: &mut Vec<UserBalance> = match balances
            .iter_mut()
            .find(|bal| bal.len() > 0 && bal[0].currency == *currency)
        {
            Some(bal) => bal,
            None => &mut empty_balances,
        };

        for spending in spending_currency {
            let balance_index = balances_currency
                .iter()
                .position(|bal| bal.username == spending.username);

            let mut paid_amount = spending.balance;
            match balance_index {
                Some(index) => {
                    paid_amount += balances_currency[index].balance;
                    balances_currency[index].balance = 0;
                }
                None => {}
            }

            let mut spending_amount = spending.balance.clone();

            if should_convert {
                spending_amount = convert_currency_with_rate(
                    spending_amount,
                    &currency,
                    &default_currency,
                    conversion_rate,
                );
                paid_amount = convert_currency_with_rate(
                    paid_amount,
                    &currency,
                    &default_currency,
                    conversion_rate,
                );
            }

            group_spending += spending_amount;

            let user = user_spendings
                .iter()
                .position(|user| user.username == spending.username);

            match user {
                Some(index) => {
                    user_spendings[index].spending += spending_amount;
                    user_spendings[index].paid += paid_amount;
                }
                None => {
                    user_spendings.push(UserSpending {
                        username: spending.username.clone(),
                        spending: spending_amount,
                        paid: paid_amount,
                    });
                }
            }
        }

        // Check through for any balances that aren't accounted for in a spending
        for balance in balances_currency {
            if balance.balance != 0 {
                let user = user_spendings
                    .iter()
                    .position(|spending| spending.username == balance.username);
                let converted_balance = convert_currency_with_rate(
                    balance.balance,
                    &currency,
                    &default_currency,
                    conversion_rate,
                );
                match user {
                    Some(index) => {
                        user_spendings[index].paid += converted_balance;
                    }
                    None => {
                        user_spendings.push(UserSpending {
                            username: balance.username.clone(),
                            spending: 0,
                            paid: converted_balance,
                        });
                    }
                }
            }
        }
    }

    Ok(SpendingData {
        currency: default_currency.to_string(),
        group_spending,
        user_spendings,
    })
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
        ChatSetting::EraseMessages(_) => {
            let erase = get_erase_messages(chat_id)?;
            Ok(ChatSetting::EraseMessages(Some(erase)))
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
                update_balances(chat_id, Vec::new())?;
            }
        }
        ChatSetting::EraseMessages(erase) => {
            if let Some(erase) = erase {
                set_erase_messages(chat_id, erase)?;
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
            let balances = get_chat_balances_currency(chat_id, CURRENCY_CODE_DEFAULT)?;
            for balance in balances {
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

            // Update all spendings to old currency
            let mut spendings_changes: Vec<UserBalance> = Vec::new();
            let spendings = retrieve_spending_data_by_currency(chat_id, CURRENCY_CODE_DEFAULT)?;
            for spending in spendings.user_spendings {
                let change_sub = UserBalance {
                    username: spending.username.clone(),
                    currency: CURRENCY_CODE_DEFAULT.to_string(),
                    balance: spending.spending.neg(),
                };
                let change_add = UserBalance {
                    username: spending.username.clone(),
                    currency: old_currency.clone(),
                    balance: spending.spending,
                };
                spendings_changes.extend(vec![change_sub, change_add]);
            }

            update_chat_spendings(chat_id, spendings_changes)?;
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
    update_balances(chat_id, changes)?;

    Ok(())
}

/* Asserts that a user has not exceeded the rate limit.
 */
pub fn assert_rate_limit(user_id: &str, timestamp: i64) -> Result<(), ProcessError> {
    let status = is_request_limit_exceeded(user_id, timestamp)?;
    if status {
        Err(ProcessError::CrudError(
            CrudError::RequestLimitExceededError(),
        ))
    } else {
        Ok(())
    }
}
