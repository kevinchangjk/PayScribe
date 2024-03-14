use redis::{Commands, Connection, RedisResult};

const USER_KEY: &str = "user";
const USER_ID_KEY: &str = "user_id";

/* user.rs contains CRUD operations for both `user` and `user_id`.
 * `user` is the main table used for normal operations.
 * `user_id` is used only to ensure the correctness of `user.username`
 */

/* User CRUD Operations
 * User represents a user, most likely in a group chat on Telegram.
 * User comprises a list of chats they are using PayScribe in.
 * Has add, exists, get, update, and delete operations.
 */

// Adds a new user to Redis
// Initialises user_id if provided
pub fn add_user(
    con: &mut Connection,
    username: &str,
    chat_id: &str,
    user_id: Option<&str>,
) -> RedisResult<()> {
    if let Some(id) = user_id {
        initialize_user(con, id, username)?;
    }

    con.rpush(&format!("{USER_KEY}:{username}"), chat_id)
}

// Checks if user exists
pub fn get_user_exists(con: &mut Connection, username: &str) -> RedisResult<bool> {
    con.exists(&format!("{USER_KEY}:{username}"))
}

// Gets user chats from a specified user
pub fn get_user_chats(con: &mut Connection, username: &str) -> RedisResult<Vec<String>> {
    con.lrange(&format!("{USER_KEY}:{username}"), 0, -1)
}

// Update user chats with a new chat
// Automatically checks if chat is already inside
pub fn update_user_chats(con: &mut Connection, username: &str, chat_id: &str) -> RedisResult<()> {
    con.rpush(&format!("{USER_KEY}:{username}"), chat_id)
}

// Deletes a user from Redis
// Mainly for testing purposes
// In application, no real need to delete keys
pub fn delete_user(con: &mut Connection, username: &str) -> RedisResult<()> {
    con.del(&format!("{USER_KEY}:{username}"))
}

/* User ID CRUD Operations
 * User ID represents a mapping of user_id to username.
 * Has add, exists, get, update, and delete operations.
 */

// Initialises user with user_id
pub fn initialize_user(con: &mut Connection, user_id: &str, username: &str) -> RedisResult<()> {
    con.set(&format!("{USER_ID_KEY}:{user_id}"), username)
}

// Checks if user is initialised
pub fn get_user_is_init(con: &mut Connection, user_id: &str) -> RedisResult<bool> {
    con.exists(&format!("{USER_ID_KEY}:{user_id}"))
}

// Gets username from a specified user_id
pub fn get_username(con: &mut Connection, user_id: &str) -> RedisResult<String> {
    con.get(&format!("{USER_ID_KEY}:{user_id}"))
}

// Updates username for a specified user_id
// Only used when user_id is provided, activated when a change in username is detected
// Otherwise, impossible to detect change in username without user_id
pub fn update_username(con: &mut Connection, user_id: &str, username: &str) -> RedisResult<()> {
    con.set(&format!("{USER_ID_KEY}:{user_id}"), username)
}

// Deletes a user_id from Redis
// Mainly for testing purposes
// In application, no real need to delete keys
pub fn delete_user_id(con: &mut Connection, user_id: &str) -> RedisResult<()> {
    con.del(&format!("{USER_ID_KEY}:{user_id}"))
}

// Tests
#[cfg(test)]
mod tests {
    use super::add_user;
    use super::delete_user;
    use super::delete_user_id;
    use super::get_user_chats;
    use super::get_user_exists;
    use super::get_user_is_init;
    use super::get_username;
    use super::initialize_user;
    use super::update_user_chats;
    use super::update_username;
    use crate::bot::redis::connect::connect;

    #[test]
    fn test_add_user_all() {
        let mut con = connect();

        let username = "test_user_all";
        let user_id = "123456789";
        let chat_id = "9876543210";
        assert!(add_user(&mut con, username, chat_id, Some(user_id),).is_ok());

        delete_user(&mut con, username).unwrap();
        delete_user_id(&mut con, user_id).unwrap();
    }

    #[test]
    fn test_add_user_no_id() {
        let mut con = connect();

        let username = "test_user_no_id";
        let chat_id = "9876543211";
        assert!(add_user(&mut con, username, chat_id, None).is_ok());

        delete_user(&mut con, username).unwrap();
    }

    #[test]
    fn test_get_user_exists_chat() {
        let mut con = connect();

        let username = "test_user_exists";
        let chat_id = "9876543212";
        add_user(&mut con, username, chat_id, None).unwrap();
        assert!(get_user_exists(&mut con, username).unwrap());
        assert!(get_user_chats(&mut con, username).unwrap() == vec![chat_id]);

        delete_user(&mut con, username).unwrap();
    }

    #[test]
    fn test_update_chats() {
        let mut con = connect();

        let username = "test_user_update_chats";
        let chat_id = "9876543214";
        let new_chat_id = "9876543215";
        add_user(&mut con, username, chat_id, None).unwrap();
        assert_eq!(get_user_chats(&mut con, username).unwrap(), vec![chat_id]);

        update_user_chats(&mut con, username, new_chat_id).unwrap();
        assert_eq!(
            get_user_chats(&mut con, username).unwrap(),
            vec![chat_id, new_chat_id]
        );

        delete_user(&mut con, username).unwrap();
    }

    #[test]
    fn test_delete_user() {
        let mut con = connect();

        let username = "test_user_delete";
        let chat_id = "9876543216";
        add_user(&mut con, username, chat_id, None).unwrap();
        assert!(get_user_exists(&mut con, username).unwrap());
        delete_user(&mut con, username).unwrap();
        assert!(!get_user_exists(&mut con, username).unwrap());
    }

    #[test]
    fn test_initialize_get_user() {
        let mut con = connect();

        let username = "test_user_initialize";
        let user_id = "1234567895";
        assert!(initialize_user(&mut con, user_id, username).is_ok());
        assert!(get_username(&mut con, user_id).unwrap() == username);
        assert!(get_user_is_init(&mut con, user_id).unwrap());

        delete_user_id(&mut con, user_id).unwrap();
    }

    #[test]
    fn test_get_user_is_not_init() {
        let mut con = connect();

        let username = "test_user_get_is_not_init";
        let chat_id = "9876543217";
        add_user(&mut con, username, chat_id, None).unwrap();
        assert!(!get_user_is_init(&mut con, username).unwrap());

        delete_user(&mut con, username).unwrap();
    }

    #[test]
    fn test_get_user_auto_init() {
        let mut con = connect();

        let username = "test_user_auto_init";
        let user_id = "1234567894";
        let chat_id = "9876543218";
        add_user(&mut con, username, chat_id, Some(user_id)).unwrap();
        assert!(get_user_is_init(&mut con, user_id).unwrap());

        delete_user_id(&mut con, user_id).unwrap();
        delete_user(&mut con, username).unwrap();
    }

    #[test]
    fn test_update_username() {
        let mut con = connect();

        let user_id = "1234567896";
        let old_username = "test_user_update_username";
        let new_username = "test_user_update_username_new";
        initialize_user(&mut con, user_id, old_username).unwrap();
        assert_eq!(get_username(&mut con, user_id).unwrap(), old_username);

        update_username(&mut con, user_id, new_username).unwrap();
        assert_eq!(get_username(&mut con, user_id).unwrap(), new_username);

        delete_user_id(&mut con, user_id).unwrap();
    }

    #[test]
    fn test_delete_user_id() {
        let mut con = connect();

        let user_id = "1234567897";
        let username = "test_user_delete_user_id";
        initialize_user(&mut con, user_id, username).unwrap();
        assert!(get_user_is_init(&mut con, user_id).unwrap());
        delete_user_id(&mut con, user_id).unwrap();
        assert!(!get_user_is_init(&mut con, user_id).unwrap());
    }
}
