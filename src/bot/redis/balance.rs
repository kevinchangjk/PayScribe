use super::connect::connect;
use redis::{Commands, RedisWrite, ToRedisArgs};

const BALANCE_KEY: &str = "balance:";

enum BalanceField {
    UserID(String),
    Amount(i32),
}

impl ToRedisArgs for BalanceField {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        match *self {
            BalanceField::UserID(ref s) => s.write_redis_args(out),
            BalanceField::Amount(ref i) => i.write_redis_args(out),
        }
    }
}

// Adds a new balance to Redis
pub fn add_balance(chat_id: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let balance: &[(&str, BalanceField)] = &[
        ("user_id", BalanceField::UserID(user_id.to_string())),
        ("amount_into", BalanceField::Amount(0)),
        ("amount_from", BalanceField::Amount(0)),
    ];
    con.hset_multiple(format!("{BALANCE_KEY}{chat_id}"), balance)
}

// Tests
#[cfg(test)]
mod tests {
    use super::add_balance;

    #[test]
    fn test_add_balance() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(add_balance(chat_id, user_id).is_ok());
    }
}
