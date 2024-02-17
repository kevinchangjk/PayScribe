// bot/mod.rs

// Re-export important items for external use
pub use self::handlers::do_action;
pub use self::handlers::Command;
// Add other re-exports as needed

// Declare submodules
mod actions;
mod handlers;
mod redis;
