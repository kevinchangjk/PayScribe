use redis::{Commands, Connection, RedisResult};

/* Spending CRUD Operations
 * Spending represents the total expenses incurred by a user in a group.
 * Has get, set, exists, and delete operations.
 */

const SPENDING_KEY: &str = "spending";

// Adds or updates a spending to Redis
pub fn set_spending(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
    balance: i64,
) -> RedisResult<()> {
    con.set(
        format!("{SPENDING_KEY}:{chat_id}:{user_id}:{currency}"),
        balance,
    )
}

// Checks if spending exists
pub fn get_spending_exists(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
) -> RedisResult<bool> {
    con.exists(format!("{SPENDING_KEY}:{chat_id}:{user_id}:{currency}"))
}

// Gets a spending
pub fn get_spending(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
) -> RedisResult<i64> {
    con.get(format!("{SPENDING_KEY}:{chat_id}:{user_id}:{currency}"))
}

// Deletes a spending in Redis
// Mainly for testing purposes
// In application, no real need to delete keys
#[allow(dead_code)]
pub fn delete_spending(
    con: &mut Connection,
    chat_id: &str,
    user_id: &str,
    currency: &str,
) -> RedisResult<()> {
    con.del(format!("{SPENDING_KEY}:{chat_id}:{user_id}:{currency}"))
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::redis::connect::connect;

    #[test]
    fn test_add_get_spending() {
        let mut con = connect().unwrap();
        let chat_id = "test_spending_chat";
        let user_id = "test_spending_user";
        let currency = "USD";
        let balance = 100;

        assert!(set_spending(&mut con, chat_id, user_id, currency, balance).is_ok());
        assert!(get_spending_exists(&mut con, chat_id, user_id, currency).unwrap());
        assert_eq!(
            get_spending(&mut con, chat_id, user_id, currency).unwrap(),
            balance
        );

        assert!(delete_spending(&mut con, chat_id, user_id, currency).is_ok());
    }

    #[test]
    fn test_update_spending() {
        let mut con = connect().unwrap();
        let chat_id = "test_spending_chat_2";
        let user_id = "test_spending_user_2";
        let currency = "USD";
        let balance = 100;

        assert!(set_spending(&mut con, chat_id, user_id, currency, balance).is_ok());

        let updated_balance = 200;
        assert!(set_spending(&mut con, chat_id, user_id, currency, updated_balance).is_ok());
        assert_eq!(
            get_spending(&mut con, chat_id, user_id, currency).unwrap(),
            updated_balance
        );

        assert!(delete_spending(&mut con, chat_id, user_id, currency).is_ok());
    }
}
