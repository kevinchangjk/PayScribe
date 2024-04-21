use redis::RedisError;

use super::{
    balance::{get_balance, get_balance_exists, set_balance},
    chat::{
        add_chat, add_chat_currency, add_chat_payment, add_chat_user_multiple, delete_chat_payment,
        get_chat_currencies, get_chat_currency_conversion, get_chat_debt,
        get_chat_default_currency, get_chat_exists, get_chat_payment_exists, get_chat_payments,
        get_chat_time_zone, get_chat_users, set_chat_currency_conversion, set_chat_debt,
        set_chat_default_currency, set_chat_time_zone, Debt,
    },
    connect::{connect, DBError},
    payment::{add_payment, delete_payment, get_payment, update_payment, Payment},
    spending::{get_spending, get_spending_exists, set_spending},
    user::{
        add_user, get_preferred_username, get_user_chats, get_user_exists, set_preferred_username,
        update_user_chats,
    },
    CURRENCY_CODE_DEFAULT,
};

#[derive(Debug, PartialEq, Clone)]
pub struct UserBalance {
    pub username: String,
    pub currency: String,
    pub balance: i64,
}

#[derive(Debug, PartialEq)]
pub struct UserPayment {
    pub chat_id: String,
    pub payment_id: String,
    pub payment: Payment,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CrudError {
    #[error("Redis operation error: {0}")]
    RedisError(RedisError),
    #[error("Redis database error: {0}")]
    DBError(DBError),
    #[error("No payments found")]
    NoPaymentsError(),
    #[error("No such payment entry found")]
    NoSuchPaymentError(),
    #[error("Spending computed to be negative")]
    NegativeSpendingError(),
}

// Implement the From trait to convert from RedisError to CrudError
impl From<RedisError> for CrudError {
    fn from(redis_error: RedisError) -> CrudError {
        CrudError::RedisError(redis_error)
    }
}

// Implement the From trait to convert from RedisError to CrudError
impl From<DBError> for CrudError {
    fn from(db_error: DBError) -> CrudError {
        CrudError::DBError(db_error)
    }
}

/* Redis Manager
 * Manager represents a module that manages all database operations.
 * No external package should call any of the database operations directly,
 * only through the manager.
 * The manager then exposes APIs for the main package to call.
 */

/* Checks if a user exists, and if not, adds them.
 * If the user exists, ensures that chats are updated. Inits user if not init.
 * Called whenever a new payment is added, and all relevant users are updated with this.
 */
pub fn update_user(username: &str, chat_id: &str, user_id: Option<&str>) -> Result<(), CrudError> {
    let mut con = connect()?;

    let user_key = username.to_lowercase();

    // Adds user if not exists
    if !get_user_exists(&mut con, &user_key)? {
        add_user(&mut con, &user_key, chat_id, user_id)?;
        set_preferred_username(&mut con, username, &user_key)?;
    }

    // Adds chat to user list if not already added
    let current_chats = get_user_chats(&mut con, &user_key)?;
    if !current_chats.contains(&chat_id.to_string()) {
        update_user_chats(&mut con, &user_key, chat_id)?;
    }

    Ok(())
}

/* Checks if a chat exists, and if not, adds it.
 * If the chat exists, ensures that it is updated with the usernames.
 * Called whenever a new payment is added.
 */
pub fn update_chat(chat_id: &str, usernames: Vec<String>) -> Result<(), CrudError> {
    let mut con = connect()?;

    // Adds chat if not exists
    if !get_chat_exists(&mut con, chat_id)? {
        add_chat(&mut con, chat_id, &usernames[0].to_lowercase())?;
        init_chat_settings(chat_id)?;
    }

    // Adds all users, automatically checked if added
    add_chat_user_multiple(
        &mut con,
        chat_id,
        usernames.iter().map(|user| user.to_lowercase()).collect(),
    )?;

    Ok(())
}

/* Initialises chat settings to default.
 */
fn init_chat_settings(chat_id: &str) -> Result<(), CrudError> {
    let mut con = connect()?;

    // Set default time zone
    let default_time_zone = "UTC";
    set_chat_time_zone(&mut con, chat_id, default_time_zone)?;

    // Set default currency
    set_chat_default_currency(&mut con, chat_id, CURRENCY_CODE_DEFAULT)?;

    // Set default currency conversion
    set_chat_currency_conversion(&mut con, chat_id, false)?;

    Ok(())
}

/* Sets time zone for a chat.
 */
pub fn set_time_zone(chat_id: &str, time_zone: &str) -> Result<(), CrudError> {
    let mut con = connect()?;

    set_chat_time_zone(&mut con, chat_id, time_zone)?;
    Ok(())
}

/* Gets time zone for a chat.
 */
pub fn get_time_zone(chat_id: &str) -> Result<String, CrudError> {
    let mut con = connect()?;

    let time_zone = get_chat_time_zone(&mut con, chat_id)?;
    match time_zone {
        Some(time_zone) => Ok(time_zone),
        None => Ok("UTC".to_string()),
    }
}

/* Sets default currency for a chat.
 */
pub fn set_default_currency(chat_id: &str, currency: &str) -> Result<(), CrudError> {
    let mut con = connect()?;

    set_chat_default_currency(&mut con, chat_id, currency)?;
    Ok(())
}

/* Gets default currency for a chat.
 */
pub fn get_default_currency(chat_id: &str) -> Result<String, CrudError> {
    let mut con = connect()?;

    let currency = get_chat_default_currency(&mut con, chat_id)?;
    match currency {
        Some(currency) => Ok(currency.to_string()),
        None => Ok(CURRENCY_CODE_DEFAULT.to_string()),
    }
}

/* Sets currency conversion for a chat.
 */
pub fn set_currency_conversion(chat_id: &str, conversion: bool) -> Result<(), CrudError> {
    let mut con = connect()?;

    set_chat_currency_conversion(&mut con, chat_id, conversion)?;
    Ok(())
}

/* Gets currency conversion for a chat.
 */
pub fn get_currency_conversion(chat_id: &str) -> Result<bool, CrudError> {
    let mut con = connect()?;

    let conversion = get_chat_currency_conversion(&mut con, chat_id)?;
    match conversion {
        Some(conversion) => Ok(conversion),
        None => Ok(false),
    }
}

/* Gets all balances for a chat.
 * Used when updating default currencies for a chat.
 */
pub fn get_chat_balances(chat_id: &str) -> Result<Vec<UserBalance>, CrudError> {
    let mut con = connect()?;

    // Retrieve all balances
    let mut balances: Vec<UserBalance> = Vec::new();
    let users = get_chat_users(&mut con, chat_id)?;
    let currencies = get_chat_currencies(&mut con, chat_id)?;
    for currency in &currencies {
        for user in &users {
            if get_balance_exists(&mut con, chat_id, user, &currency)? {
                let balance = get_balance(&mut con, chat_id, user, &currency)?;
                if balance != 0 {
                    let username = get_preferred_username(&mut con, user)?;
                    balances.push(UserBalance {
                        username,
                        currency: currency.to_string(),
                        balance,
                    });
                }
            }
        }
    }

    Ok(balances)
}

/* Updates balances for a chat based on given change amounts.
 * If balance does not exist, creates it.
 * Returns the updated balances.
 */
pub fn update_chat_balances(chat_id: &str, changes: Vec<UserBalance>) -> Result<(), CrudError> {
    let mut con = connect()?;

    // Update balances through changes
    for change in changes {
        let username = change.username.to_lowercase();
        let balance = change.balance;
        let currency = change.currency;

        // Update currency into chat first
        let currencies = get_chat_currencies(&mut con, chat_id)?;
        if !currencies.contains(&currency) {
            add_chat_currency(&mut con, chat_id, &currency)?;
        }

        // Update balance
        if let Ok(false) = get_balance_exists(&mut con, chat_id, &username, &currency) {
            set_balance(&mut con, chat_id, &username, &currency, balance)?;
        } else {
            let current_balance = get_balance(&mut con, chat_id, &username, &currency)?;
            let new_balance = current_balance + balance;
            set_balance(&mut con, chat_id, &username, &currency, new_balance)?;
        }
    }

    Ok(())
}

/* Sets the latest state of simplified debts for a chat.
 */
pub fn update_chat_debts(chat_id: &str, debts: &Vec<Debt>) -> Result<(), CrudError> {
    let mut con = connect()?;

    set_chat_debt(&mut con, chat_id, debts)?;

    Ok(())
}

/* Retrieves the latest state of simplified debts for a chat.
 */
pub fn retrieve_chat_debts(chat_id: &str) -> Result<Vec<Debt>, CrudError> {
    let mut con = connect()?;

    let debts = get_chat_debt(&mut con, chat_id)?;

    Ok(debts)
}

/* Adds a payment.
 * Sets a new key-value pair for the payment, and updates the payments list in chat.
 * Called whenever a new payment is added.
 */
pub fn add_payment_entry(chat_id: &str, payment: &Payment) -> Result<(), CrudError> {
    let mut con = connect()?;

    // Adds payment
    let payment_id = add_payment(&mut con, &payment)?;

    // Adds payment to chat
    add_chat_payment(&mut con, chat_id, &payment_id)?;

    Ok(())
}

/* Retrieves all payments for a chat and their details.
 * Called whenever a user views past payments.
 */
pub fn get_chat_payments_details(chat_id: &str) -> Result<Vec<UserPayment>, CrudError> {
    let mut con = connect()?;

    if let Err(_) = get_chat_payment_exists(&mut con, chat_id) {
        return Err(CrudError::NoPaymentsError());
    }

    let payment_ids = get_chat_payments(&mut con, chat_id)?;
    let mut payments: Vec<UserPayment> = Vec::new();

    if payment_ids.is_empty() {
        return Err(CrudError::NoPaymentsError());
    }

    for payment_id in payment_ids {
        let payment = get_payment(&mut con, &payment_id)?;
        let user_payment = UserPayment {
            chat_id: chat_id.to_string(),
            payment_id,
            payment,
        };
        payments.push(user_payment);
    }

    Ok(payments)
}

/* Retrieves a specific payment entry by ID.
 * Called when a user wants to edit or delete a payment.
 */
pub fn get_payment_entry(payment_id: &str) -> Result<Payment, CrudError> {
    let mut con = connect()?;

    let payment = get_payment(&mut con, payment_id);

    match payment {
        Err(_) => Err(CrudError::NoSuchPaymentError()),
        Ok(payment) => Ok(payment),
    }
}

/* Updates a payment entry.
 * Called when a user edits payment details.
 */
pub fn update_payment_entry(
    payment_id: &str,
    description: Option<&str>,
    creditor: Option<&str>,
    currency: Option<&str>,
    total: Option<&i64>,
    debts: Option<Vec<(String, i64)>>,
) -> Result<(), CrudError> {
    let mut con = connect()?;

    if let Err(_) = get_payment(&mut con, payment_id) {
        return Err(CrudError::NoSuchPaymentError());
    }

    // Updates payment
    update_payment(
        &mut con,
        payment_id,
        description,
        creditor,
        currency,
        total,
        debts,
    )?;

    Ok(())
}

/* Deletes a payment entry.
 * Removes the main payment entry, and also from the list in chat.
 * Called when a user wants to remove a payment.
 */
pub fn delete_payment_entry(chat_id: &str, payment_id: &str) -> Result<(), CrudError> {
    let mut con = connect()?;

    if let Err(_) = get_payment(&mut con, payment_id) {
        return Err(CrudError::NoSuchPaymentError());
    }

    delete_payment(&mut con, payment_id)?;
    delete_chat_payment(&mut con, chat_id, payment_id)?;

    Ok(())
}

/* Updates the spendings of a chat with new changes.
 * If the spending already exists, simply adds the value to the current spending.
 * Else, it creates a new key and sets the value.
 * Does not add currency. As balances and spendings are always updated together,
 * currency will be added by balances instead.
 */
pub fn update_chat_spendings(chat_id: &str, spendings: Vec<UserBalance>) -> Result<(), CrudError> {
    let mut con = connect()?;

    for spending in spendings {
        let mut amount = spending.balance;
        let is_exists =
            get_spending_exists(&mut con, chat_id, &spending.username, &spending.currency)?;
        if is_exists {
            let current_spending =
                get_spending(&mut con, chat_id, &spending.username, &spending.currency)?;
            amount += current_spending as i64;
        }

        if amount < 0 {
            return Err(CrudError::NegativeSpendingError());
        } else {
            set_spending(
                &mut con,
                chat_id,
                &spending.username,
                &spending.currency,
                amount as u64,
            )?;
        }
    }

    Ok(())
}

/* Retrieves all spendings for a chat
 * Returns a vector of vectors of UserBalance, by currency, then by user.
 */
pub fn retrieve_chat_spendings(chat_id: &str) -> Result<Vec<Vec<UserBalance>>, CrudError> {
    let mut con = connect()?;

    let mut spendings: Vec<Vec<UserBalance>> = Vec::new();
    let users = get_chat_users(&mut con, chat_id)?;
    let currencies = get_chat_currencies(&mut con, chat_id)?;

    let mut curr_index = 0;
    for currency in &currencies {
        spendings.push(Vec::new());

        for user in &users {
            if get_spending_exists(&mut con, chat_id, user, &currency)? {
                let spending = get_spending(&mut con, chat_id, user, &currency)?;
                if spending != 0 {
                    let username = get_preferred_username(&mut con, user)?;
                    spendings[curr_index].push(UserBalance {
                        username,
                        currency: currency.to_string(),
                        balance: spending as i64,
                    });
                }
            }
        }

        curr_index += 1;
    }

    Ok(spendings)
}

#[cfg(test)]
mod tests {
    use crate::bot::redis::{
        balance::delete_balance,
        chat::{
            delete_chat, delete_chat_currencies, delete_chat_debt, delete_chat_settings,
            get_chat_users,
        },
        spending::delete_spending,
        user::{delete_preferred_username, delete_user, get_preferred_username, get_user_chats},
    };

    use super::*;

    #[test]
    fn test_update_user_add_user() {
        let mut con = connect().unwrap();

        let username = "Manager_Test_User";
        let user_key = username.to_lowercase();
        let chat_id = "manager_123456789";

        // Checks that user does not exist
        assert!(!get_user_exists(&mut con, &user_key).unwrap());

        // Adds user
        assert!(update_user(username, chat_id, None).is_ok());
        assert_eq!(
            get_preferred_username(&mut con, &user_key).unwrap(),
            username
        );
        assert_eq!(get_user_chats(&mut con, &user_key).unwrap(), vec![chat_id]);

        // Performs again, nothing should happen
        assert!(update_user(username, chat_id, None).is_ok());
        assert_eq!(
            get_preferred_username(&mut con, &user_key).unwrap(),
            username
        );
        assert_eq!(get_user_chats(&mut con, &user_key).unwrap(), vec![chat_id]);

        // Deletes user
        delete_user(&mut con, &user_key).unwrap();
        delete_preferred_username(&mut con, &user_key).unwrap();
    }

    #[test]
    fn test_update_user_add_chat() {
        let mut con = connect().unwrap();

        let username = "Manager_Test_User_0";
        let user_key = username.to_lowercase();
        let chat_id = "manager_1234567890";
        let second_chat_id = "manager_1234567891";

        // Adds user and chat
        assert!(update_user(username, chat_id, None).is_ok());
        assert_eq!(get_user_chats(&mut con, &user_key).unwrap(), vec![chat_id]);

        // Calls again, adds a second chat
        assert!(update_user(username, second_chat_id, None).is_ok());
        assert_eq!(
            get_user_chats(&mut con, &user_key).unwrap(),
            vec![chat_id, second_chat_id]
        );

        // Calls again, nothing should happen
        assert!(update_user(username, second_chat_id, None).is_ok());
        assert_eq!(
            get_user_chats(&mut con, &user_key).unwrap(),
            vec![chat_id, second_chat_id]
        );

        // Deletes user
        delete_user(&mut con, &user_key).unwrap();
        delete_preferred_username(&mut con, &user_key).unwrap();
    }

    /*
    #[test]
    fn test_update_user_init_user() {
        let mut con = connect().unwrap();

        let username = "manager_test_user_1";
        let chat_id = "manager_1234567892";
        let user_id = "manager_987654321";

        // Adds user and chat, check that init works properly
        assert!(update_user(username, chat_id, Some(user_id)).is_ok());
        assert!(get_user_is_init(&mut con, user_id).unwrap());

        // Deletes user temporarily
        delete_user(&mut con, username).unwrap();
        delete_user_id(&mut con, user_id).unwrap();
        delete_preferred_username(&mut con, username).unwrap();

        // Calls again, adds user again but without ID
        assert!(update_user(username, chat_id, None).is_ok());
        assert!(!get_user_is_init(&mut con, user_id).unwrap());

        // Calls again, should init user
        assert!(update_user(username, chat_id, Some(user_id)).is_ok());
        assert!(get_user_is_init(&mut con, user_id).unwrap());

        // Deletes user
        delete_user(&mut con, username).unwrap();
        delete_user_id(&mut con, user_id).unwrap();
        delete_preferred_username(&mut con, username).unwrap();
    }

    #[test]
    fn test_update_user_update_username() {
        let mut con = connect().unwrap();

        let username = "manager_test_user_2";
        let chat_id = "manager_1234567893";
        let user_id = "manager_987654322";
        let second_username = "manager_test_user_3";

        // Adds user and chat, check that user is init
        assert!(update_user(username, chat_id, Some(user_id)).is_ok());
        assert!(get_user_is_init(&mut con, user_id).unwrap());

        // Calls again, updates username
        assert!(update_user(second_username, chat_id, Some(user_id)).is_ok());
        assert!(get_user_is_init(&mut con, user_id).unwrap());
        assert!(get_username(&mut con, user_id).unwrap() == second_username);

        // Deletes user
        delete_user(&mut con, username).unwrap();
        delete_user(&mut con, second_username).unwrap();
        delete_user_id(&mut con, user_id).unwrap();
        delete_preferred_username(&mut con, username).unwrap();
        delete_preferred_username(&mut con, second_username).unwrap();
    }
    */

    #[test]
    fn test_update_chat_add_chat_users() {
        let mut con = connect().unwrap();

        let chat_id = "manager_1234567894";
        let mut usernames = vec![
            "manager_test_user_4".to_string(),
            "manager_test_user_5".to_string(),
            "manager_test_user_6".to_string(),
        ];
        let more_usernames = vec![
            "manager_test_user_7".to_string(),
            "manager_test_user_8".to_string(),
            "manager_test_user_9".to_string(),
        ];

        // Check that chat does not exist
        assert!(!get_chat_exists(&mut con, chat_id).unwrap());

        // Add chat with first group of usernames
        assert!(update_chat(chat_id, usernames.clone()).is_ok());
        assert!(get_chat_exists(&mut con, chat_id).unwrap());
        assert_eq!(
            get_chat_users(&mut con, chat_id).unwrap(),
            vec![
                "manager_test_user_4".to_string(),
                "manager_test_user_5".to_string(),
                "manager_test_user_6".to_string(),
            ]
        );
        assert_eq!(get_time_zone(chat_id).unwrap(), "UTC".to_string());
        assert_eq!(
            get_default_currency(chat_id).unwrap(),
            CURRENCY_CODE_DEFAULT.to_string()
        );
        assert_eq!(get_currency_conversion(chat_id).unwrap(), false);

        // Call again, add both groups of usernames
        usernames.extend(more_usernames.clone());
        assert!(update_chat(chat_id, usernames.clone()).is_ok());
        assert!(get_chat_exists(&mut con, chat_id).unwrap());
        assert_eq!(
            get_chat_users(&mut con, chat_id).unwrap(),
            vec![
                "manager_test_user_4".to_string(),
                "manager_test_user_5".to_string(),
                "manager_test_user_6".to_string(),
                "manager_test_user_7".to_string(),
                "manager_test_user_8".to_string(),
                "manager_test_user_9".to_string(),
            ]
        );

        // Call again, nothing should happen
        assert!(update_chat(chat_id, usernames).is_ok());
        assert!(get_chat_exists(&mut con, chat_id).unwrap());
        assert_eq!(
            get_chat_users(&mut con, chat_id).unwrap(),
            vec![
                "manager_test_user_4".to_string(),
                "manager_test_user_5".to_string(),
                "manager_test_user_6".to_string(),
                "manager_test_user_7".to_string(),
                "manager_test_user_8".to_string(),
                "manager_test_user_9".to_string(),
            ]
        );

        // Deletes chat
        delete_chat(&mut con, chat_id).unwrap();
        delete_chat_settings(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_add_get_update_delete_payment_details() {
        let chat_id = "manager_1234567895";
        let payment = Payment {
            description: "manager_test_payment".to_string(),
            datetime: "2021-01-01T00:00:00".to_string(),
            creditor: "manager_test_user_10".to_string(),
            currency: "USD".to_string(),
            total: 10000,
            debts: vec![
                ("manager_test_user_11".to_string(), 5000),
                ("manager_test_user_12".to_string(), 5000),
            ],
        };

        // Adds payment
        assert!(add_payment_entry(chat_id, &payment).is_ok());

        let second_payment = Payment {
            description: "manager_test_payment_2".to_string(),
            datetime: "2021-01-01T00:00:01".to_string(),
            creditor: "manager_test_user_13".to_string(),
            currency: "USD".to_string(),
            total: 20000,
            debts: vec![
                ("manager_test_user_14".to_string(), 10000),
                ("manager_test_user_15".to_string(), 10000),
            ],
        };

        // Adds second payment
        assert!(add_payment_entry(chat_id, &second_payment).is_ok());

        // Gets both payments
        let payments = get_chat_payments_details(chat_id).unwrap();
        let second_id = payments[0].payment_id.clone();

        // Gets second payment by ID
        let second_payment_values = get_payment_entry(&second_id);
        assert_eq!(second_payment_values.unwrap(), second_payment);

        // Updates second payment
        let updated_description = "manager_test_payment_3";
        let updated_creditor = "manager_test_user_16";
        let updated_currency = "JPY";
        let updated_total = 30000;
        let updated_debts = vec![
            ("manager_test_user_17".to_string(), 15000),
            ("manager_test_user_18".to_string(), 15000),
        ];

        assert!(update_payment_entry(
            &second_id,
            Some(updated_description),
            Some(updated_creditor),
            Some(updated_currency),
            Some(&updated_total),
            Some(updated_debts.clone()),
        )
        .is_ok());

        // Gets both payments again
        let payments = get_chat_payments_details(chat_id).unwrap();
        assert_eq!(
            payments,
            vec![
                UserPayment {
                    chat_id: chat_id.to_string(),
                    payment_id: second_id.clone(),
                    payment: Payment {
                        description: updated_description.to_string(),
                        datetime: "2021-01-01T00:00:01".to_string(),
                        creditor: updated_creditor.to_string(),
                        currency: updated_currency.to_string(),
                        total: updated_total,
                        debts: updated_debts.clone(),
                    },
                },
                UserPayment {
                    chat_id: chat_id.to_string(),
                    payment_id: payments[1].payment_id.clone(),
                    payment: payment,
                },
            ]
        );

        // Deletes both payments
        assert!(delete_payment_entry(chat_id, &payments[0].payment_id).is_ok());
        assert!(delete_payment_entry(chat_id, &payments[1].payment_id).is_ok());
    }

    // Test for empty payments
    #[test]
    fn test_no_payments_found() {
        let chat_id = "manager_1234567898";

        let payment = Payment {
            description: "manager_test_user_20".to_string(),
            datetime: "2021-01-01T00:00:00".to_string(),
            creditor: "manager_test_user_21".to_string(),
            currency: "USD".to_string(),
            total: 10000,
            debts: vec![
                ("manager_test_user_22".to_string(), 5000),
                ("manager_test_user_23".to_string(), 5000),
            ],
        };

        // Checks that payments don't exist
        assert_eq!(
            get_chat_payments_details(chat_id).unwrap_err(),
            CrudError::NoPaymentsError()
        );

        // Adds chat payment
        assert!(add_payment_entry(chat_id, &payment).is_ok());

        // Updates fake payment, should fail
        assert_eq!(
            update_payment_entry(
                "nonexistent_payment",
                Some("manager_test_payment_3"),
                Some("manager_test_user_16"),
                Some("JPY"),
                Some(&30000),
                Some(vec![
                    ("manager_test_user_17".to_string(), 15000),
                    ("manager_test_user_18".to_string(), 15000),
                ]),
            )
            .unwrap_err(),
            CrudError::NoSuchPaymentError()
        );

        // Deletes fake payment, should fail
        assert_eq!(
            delete_payment_entry(chat_id, "nonexistent_payment").unwrap_err(),
            CrudError::NoSuchPaymentError()
        );

        // Deletes actual payment
        let payments = get_chat_payments_details(chat_id).unwrap();
        assert!(delete_payment_entry(chat_id, &payments[0].payment_id).is_ok());

        // Checks that payments don't exist
        assert_eq!(
            get_chat_payments_details(chat_id).unwrap_err(),
            CrudError::NoPaymentsError()
        );
    }

    #[test]
    fn test_update_retrieve_balances() {
        let mut con = connect().unwrap();

        let chat_id = "manager_1234567898";

        // Add users to chat
        let usernames = vec![
            "manager_test_user_20".to_string(),
            "manager_test_user_21".to_string(),
            "manager_test_user_22".to_string(),
        ];
        update_chat(chat_id, usernames.clone()).unwrap();

        for username in &usernames {
            update_user(&username, chat_id, None).unwrap();
        }

        let changes = vec![
            UserBalance {
                username: "manager_test_user_20".to_string(),
                balance: 10000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_21".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_22".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
        ];

        // Adds initial balances
        assert!(update_chat_balances(chat_id, changes.clone()).is_ok());
        let initial_balances = get_chat_balances(chat_id).unwrap();

        // Checks that balances are correct
        assert_eq!(
            initial_balances,
            vec![
                UserBalance {
                    username: "manager_test_user_20".to_string(),
                    balance: 10000,
                    currency: "USD".to_string(),
                },
                UserBalance {
                    username: "manager_test_user_21".to_string(),
                    balance: -5000,
                    currency: "USD".to_string(),
                },
                UserBalance {
                    username: "manager_test_user_22".to_string(),
                    balance: -5000,
                    currency: "USD".to_string(),
                },
            ]
        );

        // Updates balances
        let new_changes = vec![
            UserBalance {
                username: "manager_test_user_20".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_21".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_22".to_string(),
                balance: 5000,
                currency: "USD".to_string(),
            },
        ];

        assert!(update_chat_balances(chat_id, new_changes.clone()).is_ok());
        let new_balances = get_chat_balances(chat_id).unwrap();

        // Checks that balances are correct
        assert_eq!(
            new_balances,
            vec![
                UserBalance {
                    username: "manager_test_user_20".to_string(),
                    balance: 5000,
                    currency: "USD".to_string(),
                },
                UserBalance {
                    username: "manager_test_user_21".to_string(),
                    balance: -10000,
                    currency: "USD".to_string(),
                },
            ]
        );

        // Deletes balances
        for balance in initial_balances {
            delete_balance(&mut con, chat_id, &balance.username, "USD").unwrap();
        }

        // Deletes usernames
        for username in &usernames {
            delete_user(&mut con, &username).unwrap();
            delete_preferred_username(&mut con, username).unwrap();
        }

        // Deletes chat
        delete_chat(&mut con, chat_id).unwrap();
        delete_chat_currencies(&mut con, chat_id).unwrap();
        delete_chat_settings(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_multiple_currencies_balances() {
        let mut con = connect().unwrap();

        let chat_id = "manager_1234567899";

        // Add users to chat
        let usernames = vec![
            "manager_test_user_26".to_string(),
            "manager_test_user_27".to_string(),
            "manager_test_user_28".to_string(),
        ];
        update_chat(chat_id, usernames.clone()).unwrap();

        for username in &usernames {
            update_user(&username, chat_id, None).unwrap();
        }

        // Add first changes
        let changes = vec![
            UserBalance {
                username: "manager_test_user_26".to_string(),
                balance: 10000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_27".to_string(),
                balance: -5000,
                currency: "JPY".to_string(),
            },
            UserBalance {
                username: "manager_test_user_28".to_string(),
                balance: 10000,
                currency: "JPY".to_string(),
            },
        ];

        update_chat_balances(chat_id, changes.clone()).unwrap();

        // Add second changes
        let new_changes = vec![
            UserBalance {
                username: "manager_test_user_26".to_string(),
                balance: -5000,
                currency: "JPY".to_string(),
            },
            UserBalance {
                username: "manager_test_user_27".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_28".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
        ];

        assert!(update_chat_balances(chat_id, new_changes.clone()).is_ok());
        let new_balances = get_chat_balances(chat_id).unwrap();

        // Check balances
        let balances = vec![
            UserBalance {
                username: "manager_test_user_26".to_string(),
                balance: 10000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_27".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_28".to_string(),
                balance: -5000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_26".to_string(),
                balance: -5000,
                currency: "JPY".to_string(),
            },
            UserBalance {
                username: "manager_test_user_27".to_string(),
                balance: -5000,
                currency: "JPY".to_string(),
            },
            UserBalance {
                username: "manager_test_user_28".to_string(),
                balance: 10000,
                currency: "JPY".to_string(),
            },
        ];

        assert_eq!(new_balances, balances);

        // Deletes balances
        for balance in balances {
            delete_balance(&mut con, chat_id, &balance.username, &balance.currency).unwrap();
        }

        // Deletes usernames
        for username in &usernames {
            delete_user(&mut con, &username).unwrap();
            delete_preferred_username(&mut con, username).unwrap();
        }

        // Deletes chat
        delete_chat(&mut con, chat_id).unwrap();
        delete_chat_currencies(&mut con, chat_id).unwrap();
        delete_chat_settings(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_update_chat_debt() {
        let mut con = connect().unwrap();

        let chat_id = "manager_12345678990";
        let debts = vec![
            Debt {
                debtor: "manager_test_user_25".to_string(),
                creditor: "manager_test_user_26".to_string(),
                currency: "USD".to_string(),
                amount: 10000,
            },
            Debt {
                debtor: "manager_test_user_27".to_string(),
                creditor: "manager_test_user_28".to_string(),
                currency: "USD".to_string(),
                amount: 5000,
            },
        ];

        // Adds debts
        assert!(update_chat_debts(chat_id, &debts).is_ok());

        // Checks that debts are correct
        assert_eq!(retrieve_chat_debts(chat_id).unwrap(), debts);

        // Updates debts
        let new_debts = vec![
            Debt {
                debtor: "manager_test_user_25".to_string(),
                creditor: "manager_test_user_26".to_string(),
                currency: "USD".to_string(),
                amount: 5000,
            },
            Debt {
                debtor: "manager_test_user_27".to_string(),
                creditor: "manager_test_user_28".to_string(),
                currency: "USD".to_string(),
                amount: 10000,
            },
        ];
        assert!(update_chat_debts(chat_id, &new_debts).is_ok());

        // Checks that debts are correct
        assert_eq!(retrieve_chat_debts(chat_id).unwrap(), new_debts);

        // Deletes debts
        delete_chat_debt(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_get_set_chat_settings() {
        let mut con = connect().unwrap();

        let chat_id = "manager_12345678991";
        let usernames = vec![
            "manager_test_user_29".to_string(),
            "manager_test_user_30".to_string(),
            "manager_test_user_31".to_string(),
        ];

        // Checks "default" chat settings
        assert_eq!(get_time_zone(chat_id).unwrap(), "UTC".to_string());
        assert_eq!(
            get_default_currency(chat_id).unwrap(),
            CURRENCY_CODE_DEFAULT.to_string()
        );
        assert_eq!(get_currency_conversion(chat_id).unwrap(), false);

        // Adds chat
        assert!(update_chat(chat_id, usernames.clone()).is_ok());
        assert_eq!(get_time_zone(chat_id).unwrap(), "UTC".to_string());
        assert_eq!(
            get_default_currency(chat_id).unwrap(),
            CURRENCY_CODE_DEFAULT.to_string()
        );
        assert_eq!(get_currency_conversion(chat_id).unwrap(), false);

        // Sets various chat settings
        let time_zone = "GMT";
        let currency = "USD";
        let conversion = true;

        assert!(set_time_zone(chat_id, time_zone).is_ok());
        assert_eq!(get_time_zone(chat_id).unwrap(), time_zone);
        assert!(set_default_currency(chat_id, currency).is_ok());
        assert_eq!(get_default_currency(chat_id).unwrap(), currency);
        assert!(set_currency_conversion(chat_id, conversion).is_ok());
        assert_eq!(get_currency_conversion(chat_id).unwrap(), conversion);

        // Deletes chat
        delete_chat(&mut con, chat_id).unwrap();
        delete_chat_settings(&mut con, chat_id).unwrap();
    }

    #[test]
    fn tset_update_retrieve_chat_spendings() {
        let mut con = connect().unwrap();

        let chat_id = "manager_12345678992";
        let usernames = vec![
            "manager_test_user_32".to_string(),
            "manager_test_user_33".to_string(),
            "manager_test_user_34".to_string(),
        ];

        // Adds chat
        assert!(update_chat(chat_id, usernames.clone()).is_ok());

        for username in &usernames {
            update_user(&username, chat_id, None).unwrap();
        }

        // Adds spendings
        let spendings = vec![
            UserBalance {
                username: "manager_test_user_32".to_string(),
                balance: 10000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_33".to_string(),
                balance: 5000,
                currency: "USD".to_string(),
            },
            UserBalance {
                username: "manager_test_user_34".to_string(),
                balance: 3000,
                currency: "USD".to_string(),
            },
        ];

        assert!(update_chat_spendings(chat_id, spendings.clone()).is_ok());

        // Manually add currency
        assert!(add_chat_currency(&mut con, chat_id, "USD").is_ok());

        // Retrieves spendings
        let retrieved_spendings = retrieve_chat_spendings(chat_id);
        assert!(retrieved_spendings.is_ok());
        assert_eq!(retrieved_spendings.unwrap(), vec![spendings.clone()]);

        // Updates spendings with another currency
        let new_spendings = vec![
            UserBalance {
                username: "manager_test_user_32".to_string(),
                balance: 500,
                currency: "JPY".to_string(),
            },
            UserBalance {
                username: "manager_test_user_33".to_string(),
                balance: 1000,
                currency: "JPY".to_string(),
            },
            UserBalance {
                username: "manager_test_user_34".to_string(),
                balance: 1500,
                currency: "JPY".to_string(),
            },
        ];

        assert!(update_chat_spendings(chat_id, new_spendings.clone()).is_ok());

        // Manually add currency
        assert!(add_chat_currency(&mut con, chat_id, "JPY").is_ok());

        // Retrieves and checks spendings again
        let retrieved_spendings = retrieve_chat_spendings(chat_id);
        assert!(retrieved_spendings.is_ok());
        assert_eq!(
            retrieved_spendings.unwrap(),
            vec![spendings.clone(), new_spendings.clone()]
        );

        // Deletes spendings
        for spending in spendings {
            assert!(
                delete_spending(&mut con, chat_id, &spending.username, &spending.currency).is_ok()
            );
        }

        for spending in new_spendings {
            assert!(
                delete_spending(&mut con, chat_id, &spending.username, &spending.currency).is_ok()
            );
        }

        // Deletes usernames
        for username in &usernames {
            delete_user(&mut con, &username).unwrap();
            delete_preferred_username(&mut con, username).unwrap();
        }

        // Deletes chat
        delete_chat(&mut con, chat_id).unwrap();
        delete_chat_currencies(&mut con, chat_id).unwrap();
        delete_chat_settings(&mut con, chat_id).unwrap();
    }
}
