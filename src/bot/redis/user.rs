use super::connect::connect;
use redis::{Commands, RedisWrite, ToRedisArgs};

const USER_KEY: &str = "user";

enum UserFields {
    Username(String),
    IsInit(bool),
    Chats(Vec<String>),
}

impl ToRedisArgs for UserFields {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        match self {
            UserFields::Username(username) => username.write_redis_args(out),
            UserFields::IsInit(is_init) => is_init.write_redis_args(out),
            UserFields::Chats(chats) => chats.write_redis_args(out),
        }
    }
}

// Adds a new user to Redis
// Uses the user_id as the key, or if not, the username
// Adds in a chat if available
pub fn add_user(
    username: &str,
    user_id: Option<&str>,
    chat_id: Option<&str>,
) -> redis::RedisResult<()> {
    let mut con = connect();
    let user_key = match user_id {
        Some(id) => id,
        None => username,
    };
    let user: &[(&str, UserFields)] = &[
        ("username", UserFields::Username(username.to_string())),
        (
            "is_init",
            UserFields::IsInit(match user_id {
                Some(_) => true,
                None => false,
            }),
        ),
        (
            "chats",
            UserFields::Chats(match chat_id {
                Some(id) => vec![id.to_string()],
                None => vec!["".to_string()],
            }),
        ),
    ];

    con.hset_multiple(format!("{USER_KEY}:{user_key}"), user)
}

// Tests
#[cfg(test)]
mod tests {
    use super::add_user;

    #[test]
    fn test_add_user_all() {
        let username = "test_user";
        let user_id = "123456789";
        let chat_id = "987654321";
        assert!(add_user(username, Some(user_id), Some(chat_id)).is_ok());
    }

    #[test]
    fn test_add_user_no_id() {
        let username = "test_user";
        let chat_id = "987654321";
        assert!(add_user(username, None, Some(chat_id)).is_ok());
    }

    #[test]
    fn test_add_user_no_chat() {
        let username = "test_user";
        let user_id = "123456789";
        assert!(add_user(username, Some(user_id), None).is_ok());
    }

    #[test]
    fn test_add_user_no_user_id_no_chat() {
        let username = "test_user";
        assert!(add_user(username, None, None).is_ok());
    }
}
