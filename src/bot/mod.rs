// bot/mod.rs

// Re-export main functions
pub use self::handlers::do_action;

// Re-export other structs and types
pub use self::handlers::Command;

// Declare submodules
mod handlers;
mod optimizer;
mod processor;
mod redis;
