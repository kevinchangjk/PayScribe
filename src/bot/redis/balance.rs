use super::connect::connect;
use redis::Commands;

const BALANCE_KEY: &str = "balance";

// Adds a new balance to Redis
pub fn add_balance(chat_id: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let balance: &[(&str, i32)] = &[("amount_into", 0), ("amount_from", 0)];
    con.hset_multiple(format!("{BALANCE_KEY}:{chat_id}:{user_id}"), balance)
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
    use super::update_balance;

    #[test]
    fn test_add_balance() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(add_balance(chat_id, user_id).is_ok());
    }

    #[test]
    fn test_update_balance() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(update_balance(chat_id, user_id, 13, 42).is_ok());
    }
}
