use redis::RedisResult;

use super::{
    balance::{add_balance, get_balance_exists, update_balance, Balance},
    chat::{
        add_chat, add_chat_payment, add_chat_user_multiple, get_chat_exists, get_chat_payments,
    },
    connect::connect,
    payment::{add_payment, get_payment, Payment},
    user::{
        add_user, get_user_chats, get_user_exists, get_user_is_init, get_username, initialize_user,
        update_user_chats, update_username,
    },
};

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
pub fn update_user(username: &str, chat_id: &str, user_id: Option<&str>) -> RedisResult<()> {
    let mut con = connect();

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
pub fn update_chat(chat_id: &str, usernames: Vec<String>) -> RedisResult<()> {
    let mut con = connect();

    // Adds chat if not exists
    if !get_chat_exists(&mut con, chat_id)? {
        add_chat(&mut con, chat_id, &usernames[0])?;
    }

    // Adds all users, automatically checked if added
    add_chat_user_multiple(&mut con, chat_id, usernames)
}

/* Retrieves all payments for a chat and their details.
 * Called whenever a user views past payments.
 */
pub fn get_chat_payments_details(chat_id: &str) -> RedisResult<Vec<Payment>> {
    let mut con = connect();

    let payment_ids = get_chat_payments(&mut con, chat_id)?;
    let mut payments: Vec<Payment> = Vec::new();
    for payment_id in payment_ids {
        let payment = get_payment(&mut con, &payment_id)?;
        payments.push(payment);
    }
    Ok(payments)
}

/* Checks if balance for chat and user is initialized.
 * If not, adds it.
 * If yes, does nothing.
 * Basically ensures that the balance exists after the function call.
 * Called whenever a new payment is added.
 * Balance: (i32, i32), representing (amount_into, amount_from).
 */
pub fn update_balance_amounts(chat_id: &str, username: &str, balance: Balance) -> RedisResult<()> {
    let mut con = connect();

    // Adds balance if not exists
    if !get_balance_exists(&mut con, chat_id, username)? {
        add_balance(&mut con, chat_id, username)?;
    }

    update_balance(&mut con, chat_id, username, balance.0, balance.1)
}

/* Adds a payment.
 * Sets a new key-value pair for the payment, and updates the payments list in chat.
 * Called whenever a new payment is added.
 */
pub fn add_payment_entry(chat_id: &str, payment: Payment) -> RedisResult<()> {
    let mut con = connect();

    // Adds payment
    let payment_id = add_payment(&mut con, &payment)?;

    // Adds payment to chat
    add_chat_payment(&mut con, chat_id, &payment_id)
}
