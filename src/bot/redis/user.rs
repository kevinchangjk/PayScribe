use super::connect::connect;
use redis::Commands;

const USER_KEY: &str = "user";

// Adds a new user to Redis
pub fn add_user(user_id: &str, username: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let user: &[(&str, String)] = &[
        ("user_id", user_id.to_string()),
        ("username", username.to_string()),
    ];

    con.hset_multiple(format!("{USER_KEY}:{user_id}"), user)
}

// Tests
#[cfg(test)]
mod tests {
    use super::add_user;

    #[test]
    fn test_add_user() {
        let user_id = "123456789";
        let username = "test_user";
        assert!(add_user(user_id, username).is_ok());
    }
}
