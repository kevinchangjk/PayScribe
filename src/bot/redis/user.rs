use super::connect::connect;
use redis::{Commands, FromRedisValue, RedisResult, Value};

const USER_KEY: &str = "user";

pub struct User {
    username: String,
    is_init: bool,
    chats: Vec<String>,
}

impl FromRedisValue for User {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match *v {
            Value::Bulk(ref items) => {
                let mut user = User {
                    username: "".to_string(),
                    is_init: false,
                    chats: vec![],
                };
                for (i, item) in items.iter().enumerate() {
                    match i {
                        0 => user.username = String::from_redis_value(item)?,
                        1 => user.is_init = bool::from_redis_value(item)?,
                        2 => {
                            user.chats = Vec::from_redis_value(item)?;
                        }
                        _ => {}
                    }
                }
                Ok(user)
            }
            _ => Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Unknown type",
            ))),
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
    let user = User {
        username: username.to_string(),
        is_init: match user_id {
            Some(_) => true,
            None => false,
        },
        chats: match chat_id {
            Some(id) => vec![id.to_string()],
            None => vec!["".to_string()],
        },
    };

    let key = &format!("{USER_KEY}:{user_key}");
    con.hset(key, "username", &user.username)?;
    con.hset(key, "is_init", &user.is_init)?;
    con.hset(key, "chats", &user.chats)?;

    Ok(())
}

// Gets a user from Redis, can be either by user_id or username
pub fn get_user(user_key: &str) -> redis::RedisResult<User> {
    let mut con = connect();
    let username = con.hget(&format!("{USER_KEY}:{user_key}"), "username")?;
    let is_init = con.hget(&format!("{USER_KEY}:{user_key}"), "is_init")?;
    let chats = con.hget(&format!("{USER_KEY}:{user_key}"), "chats")?;
    Ok(User {
        username,
        is_init,
        chats,
    })
}

// Gets username from a specified user_id
// Only used when user_id is provided
pub fn get_username(user_id: &str) -> redis::RedisResult<String> {
    let mut con = connect();
    con.hget(&format!("{USER_KEY}:{user_id}"), "username")
}

// Checks if user is initialised
// Gets is_init from a specified user
pub fn get_user_is_init(user_key: &str) -> redis::RedisResult<bool> {
    let mut con = connect();
    con.hget(&format!("{USER_KEY}:{user_key}"), "is_init")
}

// Gets user chats from a specified user
pub fn get_user_chats(user_key: &str) -> redis::RedisResult<Vec<String>> {
    let mut con = connect();
    con.hget(&format!("{USER_KEY}:{user_key}"), "chats")
}

// Initialises user with user_id
// Only for users with current key being username
// In no other circumstance, do we delete a user
pub fn initialize_user(username: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let user = get_user(username)?;
    con.del(&format!("{USER_KEY}:{username}"))?;
    let key = &format!("{USER_KEY}:{user_id}");
    con.hset(key, "username", &user.username)?;
    con.hset(key, "is_init", true)?;
    con.hset(key, "chats", &user.chats)?;
    Ok(())
}

// Update user chats with a new chat, given a user_id or a username
pub fn update_user_chats(user_key: &str, chat_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let mut user = get_user(user_key)?;

    // Remove empty chat if it exists
    if user.chats == vec![""] {
        user.chats = vec![];
    }

    user.chats.push(chat_id.to_string());
    con.hset(&format!("{USER_KEY}:{user_key}"), "chats", &user.chats)?;
    Ok(())
}

// Updates username for a specified user_id
// Only used when user_id is provided, activated when a change in username is detected
// Otherwise, impossible to detect change in username without user_id
pub fn update_username(user_id: &str, username: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    con.hset(&format!("{USER_KEY}:{user_id}"), "username", username)
}

// Tests
#[cfg(test)]
mod tests {
    use super::add_user;
    use super::get_user;
    use super::get_user_chats;
    use super::get_user_is_init;
    use super::get_username;
    use super::initialize_user;
    use super::update_user_chats;
    use super::update_username;

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
        let user_id = "1234567890";
        assert!(add_user(username, Some(user_id), None).is_ok());
    }

    #[test]
    fn test_add_user_no_user_id_no_chat() {
        let username = "test_user";
        assert!(add_user(username, None, None).is_ok());
    }

    #[test]
    fn test_get_user() {
        let user_id = "1234567891";
        let username = "test_user";
        add_user(username, Some(&user_id), None).unwrap();
        let user = get_user(user_id).unwrap();
        assert_eq!(user.username, username);
    }

    #[test]
    fn test_get_username_by_id() {
        let user_id = "1234567892";
        let username = "test_user_get_username";
        add_user(username, Some(user_id), None).unwrap();
        assert!(get_username(user_id).unwrap() == username);
    }

    #[test]
    fn test_get_username_by_username() {
        let username = "test_user_get_username_2";
        add_user(username, None, None).unwrap();
        assert!(get_username(username).unwrap() == username);
    }

    #[test]
    fn test_get_user_is_init() {
        let user_id = "1234567893";
        let username = "test_user_get_is_init";
        add_user(username, Some(user_id), None).unwrap();
        assert!(get_user_is_init(user_id).unwrap());
    }

    #[test]
    fn test_get_user_is_not_init() {
        let username = "test_user_get_is_not_init";
        add_user(username, None, None).unwrap();
        assert!(!get_user_is_init(username).unwrap());
    }

    #[test]
    fn test_get_user_chats_empty() {
        let user_id = "1234567894";
        let username = "test_user_get_chats";
        add_user(username, Some(user_id), None).unwrap();
        assert!(get_user_chats(user_id).unwrap() == vec![""]);
    }

    #[test]
    fn test_get_user_chats() {
        let username = "test_user_get_chats_2";
        let chat = "9876543210";
        add_user(username, None, Some(chat)).unwrap();
        assert!(get_user_chats(username).unwrap() == vec![chat]);
    }

    #[test]
    fn test_initialize_user() {
        let username = "test_user_initialize";
        let user_id = "1234567895";
        add_user(username, None, None).unwrap();
        assert!(!get_user_is_init(username).unwrap());

        initialize_user(username, user_id).unwrap();
        assert!(get_user_is_init(user_id).unwrap());
    }

    #[test]
    fn test_update_chats() {
        let username = "test_user_update_chats";
        let chat_id = "9876543211";
        add_user(username, None, None).unwrap();
        assert_eq!(get_user_chats(username).unwrap(), vec![""]);

        update_user_chats(username, chat_id).unwrap();
        assert_eq!(get_user_chats(username).unwrap(), vec![chat_id]);
    }

    #[test]
    fn test_update_username() {
        let user_id = "1234567896";
        let old_username = "test_user_update_username";
        let new_username = "test_user_update_username_new";
        add_user(old_username, Some(user_id), None).unwrap();
        assert_eq!(get_username(user_id).unwrap(), old_username);

        update_username(user_id, new_username).unwrap();
        assert_eq!(get_username(user_id).unwrap(), new_username);
    }
}
