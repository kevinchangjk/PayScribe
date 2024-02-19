use redis::Commands;

const REDIS_URL: &str = "redis://127.0.0.1/";

fn connect() -> redis::Connection {
    let client = redis::Client::open(REDIS_URL).expect("Failed to connect to Redis");
    client
        .get_connection()
        .expect("Failed to get Redis connection")
}

// Tests connection to Redis
pub fn test_redis_connection() -> redis::RedisResult<isize> {
    let mut con = connect();
    let _: () = con.set("my_key", 42)?;
    con.get("my_key")
}

// Adds a new user to Redis
pub fn add_user(user_id: &str, username: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    con.set(user_id, username)
}
