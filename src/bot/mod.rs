// bot/mod.rs

// Exported functions
pub use self::dispatcher::run_dispatcher;

// Exported structs and types
pub use self::dispatcher::{Command, State};

// Declare submodules
mod currency;
mod dispatcher;
mod handler;
mod optimizer;
mod processor;
mod redis;
