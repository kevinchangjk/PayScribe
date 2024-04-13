use redis::{Commands, Connection, RedisResult};

/* Balance CRUD Operations
 * Balance represents a stake of a user in a group.
 * Balance comprises of an amount the user owes into the group, and an amount the user
 * is owed from the group.
 * Has add, exists, get, update, and delete operations.
 */

const BALANCE_KEY: &str = "balance";

// Adds or updates a balance to Redis, only called when payment is created
pub fn set_balance(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
    balance: f64,
) -> RedisResult<()> {
    con.set(
        format!("{BALANCE_KEY}:{chat_id}:{user_id}:{currency}"),
        balance,
    )
}

// Checks if balance exists
pub fn get_balance_exists(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
) -> RedisResult<bool> {
    con.exists(format!("{BALANCE_KEY}:{chat_id}:{user_id}:{currency}"))
}

// Gets a balance
pub fn get_balance(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
) -> RedisResult<f64> {
    con.get(format!("{BALANCE_KEY}:{chat_id}:{user_id}:{currency}"))
}

// Deletes a balance in Redis
// Mainly for testing purposes
// In application, no real need to delete keys
#[allow(dead_code)]
pub fn delete_balance(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
) -> RedisResult<()> {
    con.del(format!("{BALANCE_KEY}:{chat_id}:{user_id}:{currency}"))
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::redis::connect::connect;

    #[test]
    fn test_add_get_balance() {
        let mut con = connect().unwrap();

        let chat_id = "123456789";
        let user_id = "987654321";
        let currency = "USD";
        assert!(set_balance(&mut con, chat_id, user_id, currency, 13.0).is_ok());
        assert!(get_balance_exists(&mut con, chat_id, user_id, currency).unwrap());
        assert_eq!(
            get_balance(&mut con, chat_id, user_id, currency).unwrap(),
            (13.0)
        );

        delete_balance(&mut con, chat_id, user_id, currency).unwrap();
    }

    #[test]
    fn test_update_balance() {
        let mut con = connect().unwrap();

        let chat_id = "1234567891";
        let user_id = "9876543211";
        let currency = "USD";
        set_balance(&mut con, chat_id, user_id, currency, 5.0).unwrap();
        assert!(set_balance(&mut con, chat_id, user_id, currency, -42.13).is_ok());
        assert_eq!(
            get_balance(&mut con, chat_id, user_id, currency).unwrap(),
            (-42.13)
        );

        delete_balance(&mut con, chat_id, user_id, currency).unwrap();
    }

    #[test]
    fn test_delete_balance() {
        let mut con = connect().unwrap();

        let chat_id = "1234567892";
        let user_id = "9876543212";
        let currency = "SGD";
        set_balance(&mut con, chat_id, user_id, currency, 42.13).unwrap();
        assert!(delete_balance(&mut con, chat_id, user_id, currency).is_ok());
        assert!(!get_balance_exists(&mut con, chat_id, user_id, currency).unwrap());
    }
}
