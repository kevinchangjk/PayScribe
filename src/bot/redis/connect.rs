use redis::Commands;

const REDIS_URL: &str = "redis://127.0.0.1/";

pub fn connect() -> redis::Connection {
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

#[cfg(test)]
mod tests {
    use super::test_redis_connection;

    #[test]
    fn test_connection() {
        assert!(test_redis_connection().is_ok());
    }
}
