use redis::{Client, Commands, Connection, RedisError, RedisResult};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DBError {
    #[error("Redis client error: {0}")]
    RedisClientError(redis::RedisError),
    #[error("Redis connection error: {0}")]
    RedisConnectionError(redis::RedisError),
}

// Implement the From trait to convert from DBError to RedisError
impl From<DBError> for RedisError {
    fn from(db_error: DBError) -> RedisError {
        match db_error {
            DBError::RedisClientError(_) => {
                RedisError::from((redis::ErrorKind::ClientError, "Redis client error"))
            }
            DBError::RedisConnectionError(_) => {
                RedisError::from((redis::ErrorKind::IoError, "Redis connection error"))
            }
        }
    }
}

pub fn connect() -> Result<Connection, DBError> {
    dotenv::dotenv().ok();
    let url = std::env::var("REDIS_URL").expect("REDIS_URL token not set");
    match Client::open(url) {
        Ok(client) => match client.get_connection() {
            Ok(con) => Ok(con),
            Err(e) => Err(DBError::RedisConnectionError(e)),
        },
        Err(e) => Err(DBError::RedisClientError(e)),
    }
}

// Tests connection to Redis
pub fn test_redis_connection() -> RedisResult<bool> {
    let mut con = connect()?;
    con.set("my_key", 42)?;
    let res: i32 = con.get("my_key")?;
    con.del("my_key")?;

    Ok(res == 42)
}

#[cfg(test)]
mod tests {
    use super::test_redis_connection;

    // Tests working connection
    #[test]
    fn test_connection() {
        assert!(test_redis_connection().unwrap());
    }
}
