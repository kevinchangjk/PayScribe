use super::connect::connect;
use redis::Commands;

const BALANCE_KEY: &str = "balance";

// Adds a new balance to Redis
pub fn add_balance(chat_id: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let balance: &[(&str, i32)] = &[("amount_into", 0), ("amount_from", 0)];
    con.hset_multiple(format!("{BALANCE_KEY}:{chat_id}:{user_id}"), balance)
}

// Gets a balance
pub fn get_balance(chat_id: &str, user_id: &str) -> redis::RedisResult<(i32, i32)> {
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

// Tests
#[cfg(test)]
mod tests {
    use super::add_balance;
    use super::get_balance;
    use super::update_balance;

    #[test]
    fn test_add_balance() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(add_balance(chat_id, user_id).is_ok());
    }

    #[test]
    fn test_get_balance() {
        let chat_id = "1234567890";
        let user_id = "9876543210";
        add_balance(chat_id, user_id).unwrap();
        assert_eq!(get_balance(chat_id, user_id).unwrap(), (0, 0));
    }

    #[test]
    fn test_update_balance() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(update_balance(chat_id, user_id, 13, 42).is_ok());
        assert_eq!(get_balance(chat_id, user_id).unwrap(), (13, 42));
    }
}
