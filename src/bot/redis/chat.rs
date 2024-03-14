use redis::{Commands, Connection, RedisResult};

/* Chat CRUD Operations
 * Chat represents a chat, most likely a group chat on Telegram.
 * Chat comprises a list of usernames and a list of payments.
 * Has add, exists, get, update, and delete operations.
 * Except for update chat payment operation, as there is no need to do so in application.
 */

const CHAT_KEY: &str = "chat";
const CHAT_PAYMENT_KEY: &str = "chat_payment";

// Adds a new chat to Redis
pub fn add_chat(con: &mut Connection, chat_id: &str, username: &str) -> RedisResult<()> {
    con.rpush(format!("{CHAT_KEY}:{chat_id}"), username)
}

// Gets all users from a chat
// Returns a vector of usernames
pub fn get_chat_users(con: &mut Connection, chat_id: &str) -> RedisResult<Vec<String>> {
    con.lrange(format!("{CHAT_KEY}:{chat_id}"), 0, -1)
}

// Checks if chat exists
pub fn get_chat_exists(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    con.exists(format!("{CHAT_KEY}:{chat_id}"))
}

// Adds a single new user to the chat. Automatically checks if already added.
pub fn add_chat_user(con: &mut Connection, chat_id: &str, username: &str) -> RedisResult<()> {
    let current_users: Vec<String> = get_chat_users(con, chat_id)?;
    if current_users.contains(&username.to_string()) {
        return Ok(());
    }
    con.rpush(format!("{CHAT_KEY}:{chat_id}"), username)
}

// Adds more users to the chat. Automatically checks if already added.
pub fn add_chat_user_multiple(
    con: &mut Connection,
    chat_id: &str,
    users: Vec<String>,
) -> RedisResult<()> {
    let current_users: Vec<String> = get_chat_users(con, chat_id)?;
    for user in users {
        if !current_users.contains(&user) {
            con.rpush(format!("{CHAT_KEY}:{chat_id}"), user)?;
        }
    }

    Ok(())
}

// Deletes a chat from Redis
// Mainly for testing purposes
// In application, no real need to delete keys
pub fn delete_chat(con: &mut Connection, chat_id: &str) -> RedisResult<()> {
    con.del(format!("{CHAT_KEY}:{chat_id}"))
}

/* Chat Payment CRUD Operations */

// Adds a new payment to a chat
pub fn add_chat_payment(con: &mut Connection, chat_id: &str, payment_id: &str) -> RedisResult<()> {
    con.rpush(format!("{CHAT_PAYMENT_KEY}:{chat_id}"), payment_id)
}

// Checks if payments exist in a chat
pub fn get_chat_payment_exists(con: &mut Connection, chat_id: &str) -> RedisResult<bool> {
    con.exists(format!("{CHAT_PAYMENT_KEY}:{chat_id}"))
}

// Gets all payments from a chat
pub fn get_chat_payments(con: &mut Connection, chat_id: &str) -> RedisResult<Vec<String>> {
    con.lrange(format!("{CHAT_PAYMENT_KEY}:{chat_id}"), 0, -1)
}

// Deletes a payment from a chat
pub fn delete_chat_payment(
    con: &mut Connection,
    chat_id: &str,
    payment_id: &str,
) -> RedisResult<()> {
    con.lrem(format!("{CHAT_PAYMENT_KEY}:{chat_id}"), 0, payment_id)
}

// Deletes all payments from a chat
// Mainly for testing purposes
// In application, no real need to delete keys
pub fn delete_all_chat_payment(con: &mut Connection, chat_id: &str) -> RedisResult<()> {
    con.del(format!("{CHAT_PAYMENT_KEY}:{chat_id}"))
}

#[cfg(test)]
mod tests {
    use super::add_chat;
    use super::add_chat_payment;
    use super::add_chat_user;
    use super::add_chat_user_multiple;
    use super::delete_all_chat_payment;
    use super::delete_chat;
    use super::delete_chat_payment;
    use super::get_chat_exists;
    use super::get_chat_payment_exists;
    use super::get_chat_payments;
    use super::get_chat_users;
    use crate::bot::redis::connect::connect;

    #[test]
    fn test_add_chat() {
        let mut con = connect();

        let chat_id = "123456789";
        let username = "987654321";
        assert!(add_chat(&mut con, chat_id, username).is_ok());

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_get_chat_exists() {
        let mut con = connect();

        let chat_id = "1234567891";
        let username = "9876543211";
        add_chat(&mut con, chat_id, username).unwrap();
        assert!(get_chat_exists(&mut con, chat_id).unwrap());

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_get_chat_users() {
        let mut con = connect();

        let chat_id = "1234567890";
        let username = "9876543210";
        add_chat(&mut con, chat_id, username).unwrap();
        let users = get_chat_users(&mut con, chat_id);
        assert!(users.is_ok());
        assert_eq!(users.unwrap(), vec![username.to_string()]);

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_add_user_to_chat() {
        let mut con = connect();

        let chat_id = "1234567892";
        let username = "9876543212";
        let new_username = "9876543213";
        add_chat(&mut con, chat_id, username).unwrap();
        assert!(add_chat_user(&mut con, chat_id, new_username).is_ok());

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_add_users_to_chat() {
        let mut con = connect();

        let chat_id = "1234567893";
        let first_user = "987654321";
        let users = vec![
            "987654322".to_string(),
            "987654323".to_string(),
            "987654324".to_string(),
        ];
        add_chat(&mut con, chat_id, first_user).unwrap();
        assert!(add_chat_user_multiple(&mut con, chat_id, users).is_ok());
        assert_eq!(
            get_chat_users(&mut con, chat_id).unwrap(),
            vec![
                "987654321".to_string(),
                "987654322".to_string(),
                "987654323".to_string(),
                "987654324".to_string(),
            ]
        );

        delete_chat(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_delete_chat() {
        let mut con = connect();

        let chat_id = "1234567894";
        let username = "9876543216";
        add_chat(&mut con, chat_id, username).unwrap();
        assert!(get_chat_exists(&mut con, chat_id).unwrap());
        delete_chat(&mut con, chat_id).unwrap();
        assert!(!get_chat_exists(&mut con, chat_id).unwrap());
    }

    #[test]
    fn test_add_get_chat_payment() {
        let mut con = connect();

        let chat_id = "1234567895";
        let payment_id = "payment_id_1";
        assert!(add_chat_payment(&mut con, chat_id, payment_id).is_ok());
        assert!(get_chat_payment_exists(&mut con, chat_id).is_ok());
        assert!(get_chat_payments(&mut con, chat_id).unwrap() == vec![payment_id]);
        delete_all_chat_payment(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_delete_chat_payment() {
        let mut con = connect();

        let chat_id = "1234567896";
        let payment_id = "payment_id_2";
        add_chat_payment(&mut con, chat_id, payment_id).unwrap();
        let payment_id_second = "payment_id_3";
        add_chat_payment(&mut con, chat_id, payment_id_second).unwrap();
        let payment_id_third = "payment_id_4";
        add_chat_payment(&mut con, chat_id, payment_id_third).unwrap();
        delete_chat_payment(&mut con, chat_id, payment_id_second).unwrap();

        assert_eq!(
            get_chat_payments(&mut con, chat_id).unwrap(),
            vec![payment_id, payment_id_third]
        );
        delete_all_chat_payment(&mut con, chat_id).unwrap();
    }

    #[test]
    fn test_delete_all_chat_payment() {
        let mut con = connect();

        let chat_id = "1234567897";
        let payment_id = "payment_id_5";
        add_chat_payment(&mut con, chat_id, payment_id).unwrap();
        delete_all_chat_payment(&mut con, chat_id).unwrap();
        assert!(!get_chat_payment_exists(&mut con, chat_id).unwrap());
    }
}
