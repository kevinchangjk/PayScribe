// Exported functions
pub use self::connect::test_redis_connection;
pub use self::user::add_user;

// Submodules
mod connect;
mod user;
