// bot/mod.rs

// Exported functions
pub use self::handler::run_dispatcher;

// Declare submodules
mod handler;
mod optimizer;
mod processor;
mod redis;
