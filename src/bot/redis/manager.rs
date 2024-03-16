use redis::RedisError;

use super::{
    balance::{get_balance, get_balance_exists, set_balance},
    chat::{
        add_chat, add_chat_payment, add_chat_user_multiple, delete_chat_payment, get_chat_exists,
        get_chat_payment_exists, get_chat_payments, get_chat_users,
    },
    connect::{connect, DBError},
    payment::{add_payment, delete_payment, get_payment, update_payment, Payment},
    user::{
        add_user, get_user_chats, get_user_exists, get_user_is_init, get_username, initialize_user,
        update_user_chats, update_username,
    },
};

#[derive(Debug, PartialEq)]
pub struct UserPayment {
    pub chat_id: String,
    pub payment_id: String,
    pub payment: Payment,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CrudError {
    #[error("Redis operation error: {0}")]
    RedisError(redis::RedisError),
    #[error("Redis database error: {0}")]
    DBError(DBError),
    #[error("No payments found")]
    NoPaymentsError(),
    #[error("No balances found")]
    NoBalancesError(),
    #[error("No such payment entry found")]
    NoSuchPaymentError(),
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

    // Adds user if not exists
    if !get_user_exists(&mut con, username)? {
        add_user(&mut con, username, chat_id, user_id)?;
    }

    // Adds chat to user list if not already added
    let current_chats = get_user_chats(&mut con, username)?;
    if !current_chats.contains(&chat_id.to_string()) {
        update_user_chats(&mut con, username, chat_id)?;
    }

    // If user is init, ensure username is updated, else init user
    if let Some(id) = user_id {
        if !get_user_is_init(&mut con, id)? {
            initialize_user(&mut con, id, username)?;
        } else if get_username(&mut con, id)? != username {
            update_username(&mut con, id, username)?;
        }
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
        add_chat(&mut con, chat_id, &usernames[0])?;
    }

    // Adds all users, automatically checked if added
    add_chat_user_multiple(&mut con, chat_id, usernames)?;

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
        log::info!("No payments found for chat {}", chat_id);
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

/* Checks if balance for chat and user is initialized.
 * If not, adds it.
 * If yes, updates it.
 * Basically ensures that the balance exists after the function call.
 * Called whenever a new payment is added.
 */
pub fn update_balance_amounts(
    chat_id: &str,
    username: &str,
    balance: f64,
) -> Result<(), CrudError> {
    let mut con = connect()?;

    set_balance(&mut con, chat_id, username, balance)?;

    Ok(())
}

/* Retrieves balances of all users in a chat.
 * Called whenever a user wants to view current balances.
 */
pub fn retrieve_all_balances(chat_id: &str) -> Result<Vec<(String, f64)>, CrudError> {
    let mut con = connect()?;

    let usernames = get_chat_users(&mut con, chat_id)?;
    let mut balances: Vec<(String, f64)> = Vec::new();
    for username in usernames {
        if let Ok(false) = get_balance_exists(&mut con, chat_id, &username) {
            continue;
        }
        let balance = get_balance(&mut con, chat_id, &username)?;
        balances.push((username, balance));
    }

    if balances.is_empty() {
        log::info!("No balances found for chat {}", chat_id);
        return Err(CrudError::NoBalancesError());
    }

    Ok(balances)
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

/* Updates a payment entry.
 * Called when a user edits payment details.
 */
pub fn update_payment_entry(
    payment_id: &str,
    description: Option<&str>,
    creditor: Option<&str>,
    total: Option<&f64>,
    debts: Option<Vec<(String, f64)>>,
) -> Result<(), CrudError> {
    let mut con = connect()?;

    if let Err(_) = get_payment(&mut con, payment_id) {
        log::info!("No such payment found for payment_id {}", payment_id);
        return Err(CrudError::NoSuchPaymentError());
    }

    // Updates payment
    update_payment(&mut con, payment_id, description, creditor, total, debts)?;

    Ok(())
}

/* Deletes a payment entry.
 * Removes the main payment entry, and also from the list in chat.
 * Called when a user wants to remove a payment.
 */
pub fn delete_payment_entry(chat_id: &str, payment_id: &str) -> Result<(), CrudError> {
    let mut con = connect()?;

    if let Err(_) = get_payment(&mut con, payment_id) {
        log::info!("No such payment found for payment_id {}", payment_id);
        return Err(CrudError::NoSuchPaymentError());
    }

    delete_payment(&mut con, payment_id)?;
    delete_chat_payment(&mut con, chat_id, payment_id)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::bot::redis::{
        balance::delete_balance,
        chat::delete_chat,
        user::{delete_user, delete_user_id, get_user_chats},
    };

    use super::*;

    #[test]
    fn test_update_user_add_user() {
        let mut con = connect().unwrap();

        let username = "manager_test_user";
        let chat_id = "manager_123456789";

        // Checks that user does not exist
        assert!(!get_user_exists(&mut con, username).unwrap());

        // Adds user
        assert!(update_user(username, chat_id, None).is_ok());
        assert!(!get_user_is_init(&mut con, username).unwrap());
        assert_eq!(get_user_chats(&mut con, username).unwrap(), vec![chat_id]);

        // Performs again, nothing should happen
        assert!(update_user(username, chat_id, None).is_ok());
        assert!(!get_user_is_init(&mut con, username).unwrap());
        assert_eq!(get_user_chats(&mut con, username).unwrap(), vec![chat_id]);

        // Deletes user
        delete_user(&mut con, username).unwrap();
    }

    #[test]
    fn test_update_user_add_chat() {
        let mut con = connect().unwrap();

        let username = "manager_test_user_0";
        let chat_id = "manager_1234567890";
        let second_chat_id = "manager_1234567891";

        // Adds user and chat
        assert!(update_user(username, chat_id, None).is_ok());
        assert_eq!(get_user_chats(&mut con, username).unwrap(), vec![chat_id]);

        // Calls again, adds a second chat
        assert!(update_user(username, second_chat_id, None).is_ok());
        assert_eq!(
            get_user_chats(&mut con, username).unwrap(),
            vec![chat_id, second_chat_id]
        );

        // Calls again, nothing should happen
        assert!(update_user(username, second_chat_id, None).is_ok());
        assert_eq!(
            get_user_chats(&mut con, username).unwrap(),
            vec![chat_id, second_chat_id]
        );

        // Deletes user
        delete_user(&mut con, username).unwrap();
    }

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

        // Calls again, adds user again but without ID
        assert!(update_user(username, chat_id, None).is_ok());
        assert!(!get_user_is_init(&mut con, user_id).unwrap());

        // Calls again, should init user
        assert!(update_user(username, chat_id, Some(user_id)).is_ok());
        assert!(get_user_is_init(&mut con, user_id).unwrap());

        // Deletes user
        delete_user(&mut con, username).unwrap();
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
        delete_user(&mut con, second_username).unwrap();
    }

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
    }

    #[test]
    fn test_add_get_update_delete_payment_details() {
        let chat_id = "manager_1234567895";
        let payment = Payment {
            description: "manager_test_payment".to_string(),
            datetime: "2021-01-01T00:00:00".to_string(),
            creditor: "manager_test_user_10".to_string(),
            total: 100.0,
            debts: vec![
                ("manager_test_user_11".to_string(), 50.0),
                ("manager_test_user_12".to_string(), 50.0),
            ],
        };

        // Adds payment
        assert!(add_payment_entry(chat_id, &payment).is_ok());

        let second_payment = Payment {
            description: "manager_test_payment_2".to_string(),
            datetime: "2021-01-01T00:00:01".to_string(),
            creditor: "manager_test_user_13".to_string(),
            total: 200.0,
            debts: vec![
                ("manager_test_user_14".to_string(), 100.0),
                ("manager_test_user_15".to_string(), 100.0),
            ],
        };

        // Adds second payment
        assert!(add_payment_entry(chat_id, &second_payment).is_ok());

        // Gets both payments
        let payments = get_chat_payments_details(chat_id).unwrap();
        let second_id = payments[1].payment_id.clone();

        // Updates second payment
        let updated_description = "manager_test_payment_3";
        let updated_creditor = "manager_test_user_16";
        let updated_total = 300.0;
        let updated_debts = vec![
            ("manager_test_user_17".to_string(), 150.0),
            ("manager_test_user_18".to_string(), 150.0),
        ];

        assert!(update_payment_entry(
            &second_id,
            Some(updated_description),
            Some(updated_creditor),
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
                    payment_id: payments[0].payment_id.clone(),
                    payment: payment,
                },
                UserPayment {
                    chat_id: chat_id.to_string(),
                    payment_id: second_id.clone(),
                    payment: Payment {
                        description: updated_description.to_string(),
                        datetime: "2021-01-01T00:00:01".to_string(),
                        creditor: updated_creditor.to_string(),
                        total: updated_total,
                        debts: updated_debts.clone(),
                    },
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
        let username = "manager_test_user_20";

        let payment = Payment {
            description: "manager_test_user_20".to_string(),
            datetime: "2021-01-01T00:00:00".to_string(),
            creditor: "manager_test_user_21".to_string(),
            total: 100.0,
            debts: vec![
                ("manager_test_user_22".to_string(), 50.0),
                ("manager_test_user_23".to_string(), 50.0),
            ],
        };

        // Checks that payments don't exist
        assert_eq!(
            get_chat_payments_details(chat_id).unwrap_err(),
            CrudError::NoPaymentsError()
        );

        // Adds chat payment
        assert!(update_chat(chat_id, vec![username.to_string()]).is_ok());
        assert!(add_payment_entry(chat_id, &payment).is_ok());

        // Updates fake payment, should fail
        assert_eq!(
            update_payment_entry(
                "nonexistent_payment",
                Some("manager_test_payment_3"),
                Some("manager_test_user_16"),
                Some(&300.0),
                Some(vec![
                    ("manager_test_user_17".to_string(), 150.0),
                    ("manager_test_user_18".to_string(), 150.0),
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
    fn test_update_balance() {
        let mut con = connect().unwrap();

        let chat_id = "manager_1234567896";
        let username = "manager_test_user_16";
        let balance = 50.50;

        // Checks that balance doesn't exist
        assert!(!get_balance_exists(&mut con, chat_id, username).unwrap());

        // Adds initial balance
        assert!(update_balance_amounts(chat_id, username, balance).is_ok());
        assert_eq!(get_balance(&mut con, chat_id, username).unwrap(), balance);

        let second_balance = 100.4;

        // Updates balance
        assert!(update_balance_amounts(chat_id, username, second_balance).is_ok());
        assert_eq!(
            get_balance(&mut con, chat_id, username).unwrap(),
            second_balance
        );

        // Deletes balance
        delete_balance(&mut con, chat_id, username).unwrap();
    }

    #[test]
    fn test_retrieve_all_balances() {
        let mut con = connect().unwrap();

        let chat_id = "manager_1234567897";
        let usernames = vec![
            "manager_test_user_17".to_string(),
            "manager_test_user_18".to_string(),
            "manager_test_user_19".to_string(),
        ];

        // Adds chat with group of usernames
        assert!(update_chat(chat_id, usernames.clone()).is_ok());

        // Adds balances
        let first_balance = 50.0;
        let second_balance = 100.1;
        let third_balance = -100.2;

        assert!(update_balance_amounts(chat_id, &usernames[0], first_balance).is_ok());
        assert!(update_balance_amounts(chat_id, &usernames[1], second_balance).is_ok());
        assert!(update_balance_amounts(chat_id, &usernames[2], third_balance).is_ok());

        // Checks all balances
        assert_eq!(
            retrieve_all_balances(chat_id).unwrap(),
            vec![
                ("manager_test_user_17".to_string(), first_balance),
                ("manager_test_user_18".to_string(), second_balance),
                ("manager_test_user_19".to_string(), third_balance),
            ]
        );

        // Deletes all key-value pairs
        for username in usernames {
            delete_user(&mut con, &username).unwrap();
            delete_balance(&mut con, chat_id, &username).unwrap();
        }

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_no_balances_found() {
        let chat_id = "manager_1234567899";
        let username = "manager_test_user_24";

        // Checks that balances don't exist
        assert_eq!(
            retrieve_all_balances(chat_id).unwrap_err(),
            CrudError::NoBalancesError()
        );

        // Adds chat with username
        assert!(update_chat(chat_id, vec![username.to_string()]).is_ok());

        // Checks that balances still don't exist
        assert_eq!(
            retrieve_all_balances(chat_id).unwrap_err(),
            CrudError::NoBalancesError()
        );
    }
}
