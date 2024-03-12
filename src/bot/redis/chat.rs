use super::connect::connect;
use redis::Commands;

const CHAT_KEY: &str = "chat";

// Adds a new chat to Redis
pub fn add_chat(chat_id: &str, username: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    con.rpush(format!("{CHAT_KEY}:{chat_id}"), username)
}

// Gets all users from a chat
// Returns a vector of usernames
pub fn get_chat_users(chat_id: &str) -> redis::RedisResult<Vec<String>> {
    let mut con = connect();
    con.lrange(format!("{CHAT_KEY}:{chat_id}"), 0, -1)
}

// Checks if chat exists
pub fn get_chat_exists(chat_id: &str) -> redis::RedisResult<bool> {
    let mut con = connect();
    con.exists(format!("{CHAT_KEY}:{chat_id}"))
}

// Adds a single new user to the chat
pub fn add_chat_user(chat_id: &str, username: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let current_users: Vec<String> = get_chat_users(chat_id)?;
    if current_users.contains(&username.to_string()) {
        return Ok(());
    }
    con.rpush(format!("{CHAT_KEY}:{chat_id}"), username)
}

// Adds more users to the chat
// Users can be either user_id or username
pub fn add_chat_user_multiple(chat_id: &str, users: Vec<String>) -> redis::RedisResult<()> {
    let mut con = connect();
    let current_users: Vec<String> = get_chat_users(chat_id)?;
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
pub fn delete_chat(chat_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    con.del(format!("{CHAT_KEY}:{chat_id}"))
}

#[cfg(test)]
mod tests {
    use super::add_chat;
    use super::add_chat_user;
    use super::add_chat_user_multiple;
    use super::delete_chat;
    use super::get_chat_exists;
    use super::get_chat_users;

    #[test]
    fn test_add_chat() {
        let chat_id = "123456789";
        let username = "987654321";
        assert!(add_chat(chat_id, username).is_ok());
        delete_chat(chat_id).unwrap();
    }

    #[test]
    fn test_get_chat_exists() {
        let chat_id = "1234567891";
        let username = "9876543211";
        add_chat(chat_id, username).unwrap();
        assert!(get_chat_exists(chat_id).unwrap());
        delete_chat(chat_id).unwrap();
    }

    #[test]
    fn test_get_chat_users() {
        let chat_id = "1234567890";
        let username = "9876543210";
        add_chat(chat_id, username).unwrap();
        let users = get_chat_users(chat_id);
        assert!(users.is_ok());
        assert_eq!(users.unwrap(), vec![username.to_string()]);
        delete_chat(chat_id).unwrap();
    }

    #[test]
    fn test_add_user_to_chat() {
        let chat_id = "1234567892";
        let username = "9876543212";
        let new_username = "9876543213";
        add_chat(chat_id, username).unwrap();
        assert!(add_chat_user(chat_id, new_username).is_ok());
        delete_chat(chat_id).unwrap();
    }

    #[test]
    fn test_add_users_to_chat() {
        let chat_id = "1234567893";
        let first_user = "987654321";
        let users = vec![
            "987654322".to_string(),
            "987654323".to_string(),
            "987654324".to_string(),
        ];
        add_chat(chat_id, first_user).unwrap();
        assert!(add_chat_user_multiple(chat_id, users).is_ok());
        assert_eq!(
            get_chat_users(chat_id).unwrap(),
            vec![
                "987654321".to_string(),
                "987654322".to_string(),
                "987654323".to_string(),
                "987654324".to_string(),
            ]
        );
        delete_chat(chat_id).unwrap();
    }

    #[test]
    fn test_delete_chat() {
        let chat_id = "1234567894";
        let username = "9876543216";
        add_chat(chat_id, username).unwrap();
        assert!(get_chat_exists(chat_id).unwrap());
        delete_chat(chat_id).unwrap();
        assert!(!get_chat_exists(chat_id).unwrap());
    }
}
