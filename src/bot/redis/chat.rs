use super::connect::connect;
use redis::Commands;

const CHAT_KEY: &str = "chat";

// Adds a new chat to Redis
// Used only when a user commands the bot in a group chat, thus user_id is provided
pub fn add_chat(chat_id: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let chat: &[String; 1] = &[user_id.to_string()];
    con.set(format!("{CHAT_KEY}:{chat_id}"), chat)
}

// Gets all users from a chat
// Returns a vector of user_id or username
pub fn get_chat_users(chat_id: &str) -> redis::RedisResult<Vec<String>> {
    let mut con = connect();
    con.get(format!("{CHAT_KEY}:{chat_id}"))
}

// Adds a single new user to the chat
// User can be either user_id or username
pub fn add_chat_user(chat_id: &str, user: String) -> redis::RedisResult<()> {
    let mut con = connect();
    let current_users: Vec<String> = get_chat_users(chat_id)?;
    let mut new_users = current_users.clone();
    if !current_users.contains(&user) {
        new_users.push(user);
    }
    con.set(format!("{CHAT_KEY}:{chat_id}"), current_users)
}

// Adds more users to the chat
// Users can be either user_id or username
pub fn add_chat_user_multiple(chat_id: &str, users: Vec<String>) -> redis::RedisResult<()> {
    let mut con = connect();
    let current_users: Vec<String> = get_chat_users(chat_id)?;
    let mut new_users = current_users.clone();
    for user in users {
        if !current_users.contains(&user) {
            new_users.push(user);
        }
    }
    con.set(format!("{CHAT_KEY}:{chat_id}"), current_users)
}

// Updates a chat for the initialization of a user
// Used only in conjunction with user::initialize_user
pub fn update_chat_user_init(
    chat_id: &str,
    username: &str,
    user_id: &str,
) -> redis::RedisResult<()> {
    let mut con = connect();
    let current_users: Vec<String> = get_chat_users(chat_id)?;
    let mut new_users = current_users.clone();
    if current_users.contains(&username.to_string()) {
        new_users.retain(|x| x != username);
        new_users.push(user_id.to_string());
    }
    con.set(format!("{CHAT_KEY}:{chat_id}"), new_users)
}

#[cfg(test)]
mod tests {
    use super::add_chat;
    use super::add_chat_user;
    use super::add_chat_user_multiple;
    use super::get_chat_users;
    use super::update_chat_user_init;

    #[test]
    fn test_add_chat() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(add_chat(chat_id, user_id).is_ok());
    }

    #[test]
    fn test_get_chat_users() {
        let chat_id = "123456789";
        let users = get_chat_users(chat_id);
        assert!(users.is_ok());
        assert_eq!(users.unwrap(), vec!["987654321".to_string()]);
    }

    #[test]
    fn test_add_user_to_chat() {
        let chat_id = "123456789";
        let user_id = "987654325";
        assert!(add_chat_user(chat_id, user_id.to_string()).is_ok());
    }

    #[test]
    fn test_add_users_to_chat() {
        let chat_id = "123456789";
        let users = vec![
            "987654322".to_string(),
            "987654323".to_string(),
            "987654324".to_string(),
        ];
        assert!(add_chat_user_multiple(chat_id, users).is_ok());
    }

    #[test]
    fn test_update_chat_user_init() {
        let chat_id = "1234567890";
        let username = "test_user";
        let user_id = "9876543210";
        add_chat(chat_id, username).unwrap();

        update_chat_user_init(chat_id, username, user_id).unwrap();
        assert_eq!(get_chat_users(chat_id).unwrap(), vec![user_id.to_string()]);
    }
}
