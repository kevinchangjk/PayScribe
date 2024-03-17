// bot/mod.rs

// Exported functions
pub use self::handler::handler;

// Exported types
pub use self::handler::State;

// Declare submodules
mod handler;
mod optimizer;
mod processor;
mod redis;
