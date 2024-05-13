use redis::{Commands, Connection, RedisResult};

const REQUEST_KEY: &str = "request";

/* request.rs contains CRUD operations for `request`.
 * `request` is the main table used for tracking user requests.
 */

/* Request CRUD Operations
 * Request represents the counter object for a user ID, and
 * contains only the timestamp of the latest request.
 * This is because PayScribe will only accept one request per user per second.
 * Has set, get, and delete operations.
 */

// Sets a request for a user
pub fn set_request(con: &mut Connection, user_id: &str, timestamp: i64) -> RedisResult<()> {
    con.set(format!("{REQUEST_KEY}:{user_id}"), timestamp)
}

// Gets the request for a user
pub fn get_request(con: &mut Connection, user_id: &str) -> RedisResult<i64> {
    con.get(format!("{REQUEST_KEY}:{user_id}"))
}

// Deletes the request for a user
// Mainly for testing purposes
// In application, no real need to delete keys
#[allow(dead_code)]
pub fn delete_request(con: &mut Connection, user_id: &str) -> RedisResult<()> {
    con.del(format!("{REQUEST_KEY}:{user_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::redis::connect::connect;

    #[test]
    fn test_get_set_request() {
        let mut con = connect().unwrap();

        let user_id = "12345678901";
        let timestamp = 1234567890;

        assert!(set_request(&mut con, user_id, timestamp).is_ok());
        assert_eq!(get_request(&mut con, user_id).unwrap(), timestamp);

        assert!(delete_request(&mut con, user_id).is_ok());
    }
}
