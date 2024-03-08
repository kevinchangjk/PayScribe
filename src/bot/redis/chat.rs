use super::connect::connect;
use redis::Commands;

const CHAT_KEY: &str = "chat:";

// Adds a new chat to Redis
// TODO: Change to holding just an array of users, take out the chat_id
pub fn add_chat(chat_id: &str, user_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let chat: &[(&str, &[String; 1])] = &[
        ("chat_id", &[chat_id.to_string()]),
        ("users", &[user_id.to_string()]),
    ];
    con.hset_multiple(format!("{CHAT_KEY}{chat_id}"), chat)
}

#[cfg(test)]
mod tests {
    use super::add_chat;

    #[test]
    fn test_add_chat() {
        let chat_id = "123456789";
        let user_id = "987654321";
        assert!(add_chat(chat_id, user_id).is_ok());
    }
}
