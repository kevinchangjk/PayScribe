use super::connect::connect;
use redis::Commands;

/* Balance CRUD Operations
 * Balance represents a stake of a user in a group.
 * Balance comprises of an amount the user owes into the group, and an amount the user
 * is owed from the group.
 * Has add, exists, get, update, and delete operations.
 */

const BALANCE_KEY: &str = "balance";

type Balance = (i32, i32);

// Adds a new balance to Redis
pub fn add_balance(chat_id: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let balance: &[(&str, i32)] = &[("amount_into", 0), ("amount_from", 0)];
    con.hset_multiple(format!("{BALANCE_KEY}:{chat_id}:{user_id}"), balance)
}

// Checks if balance exists
pub fn get_balance_exists(chat_id: &str, user_id: &str) -> redis::RedisResult<bool> {
    let mut con = connect();
    con.exists(format!("{BALANCE_KEY}:{chat_id}:{user_id}"))
}

// Gets a balance
pub fn get_balance(chat_id: &str, user_id: &str) -> redis::RedisResult<Balance> {
    let mut con = connect();
    let amount_into: i32 = con.hget(format!("{BALANCE_KEY}:{chat_id}:{user_id}"), "amount_into")?;
    let amount_from: i32 = con.hget(format!("{BALANCE_KEY}:{chat_id}:{user_id}"), "amount_from")?;
    Ok((amount_into, amount_from))
}

// Updates a balance in Redis
pub fn update_balance(
    chat_id: &str,
    user_id: &str,
    amount_into: i32,
    amount_from: i32,
) -> redis::RedisResult<()> {
    let mut con = connect();
    let updated_balance: &[(&str, i32)] =
        &[("amount_into", amount_into), ("amount_from", amount_from)];
    con.hset_multiple(
        format!("{BALANCE_KEY}:{chat_id}:{user_id}"),
        updated_balance,
    )
}

// Deletes a balance in Redis
// Mainly for testing purposes
// In application, no real need to delete keys
pub fn delete_balance(chat_id: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    con.del(format!("{BALANCE_KEY}:{chat_id}:{user_id}"))
}

// Tests
#[cfg(test)]
mod tests {
    use super::add_balance;
    use super::delete_balance;
    use super::get_balance;
    use super::get_balance_exists;
    use super::update_balance;

    #[test]
    fn test_add_get_balance() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(add_balance(chat_id, user_id).is_ok());
        assert!(get_balance_exists(chat_id, user_id).unwrap());
        assert_eq!(get_balance(chat_id, user_id).unwrap(), (0, 0));

        delete_balance(chat_id, user_id).unwrap();
    }

    #[test]
    fn test_update_balance() {
        let chat_id = "1234567891";
        let user_id = "9876543211";
        add_balance(chat_id, user_id).unwrap();
        assert!(update_balance(chat_id, user_id, 13, 42).is_ok());
        assert_eq!(get_balance(chat_id, user_id).unwrap(), (13, 42));

        delete_balance(chat_id, user_id).unwrap();
    }

    #[test]
    fn test_delete_balance() {
        let chat_id = "1234567892";
        let user_id = "9876543212";
        add_balance(chat_id, user_id).unwrap();
        assert!(delete_balance(chat_id, user_id).is_ok());
        assert!(!get_balance_exists(chat_id, user_id).unwrap());
    }
}
