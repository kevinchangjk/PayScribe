use redis::{Client, Commands, Connection, RedisResult};

const REDIS_URL: &str = "redis://127.0.0.1/";

pub fn connect() -> Connection {
    let client = Client::open(REDIS_URL).expect("Failed to connect to Redis");
    client
        .get_connection()
        .expect("Failed to get Redis connection")
}

// Tests connection to Redis
pub fn test_redis_connection() -> RedisResult<bool> {
    let mut con = connect();
    let _: () = con.set("my_key", 42)?;
    let res: i32 = con.get("my_key")?;
    con.del("my_key")?;

    Ok(res == 42)
}

#[cfg(test)]
mod tests {
    use super::test_redis_connection;

    #[test]
    fn test_connection() {
        assert!(test_redis_connection().unwrap());
    }
}
