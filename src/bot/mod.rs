// bot/mod.rs

// Exported functions
pub use self::dispatcher::run_dispatcher;

// Exported structs and types
pub use self::dispatcher::{
    AddPaymentEdit, AddPaymentParams, Command, HandlerResult, State, UserDialogue,
};

// Declare submodules
mod dispatcher;
mod handler;
mod optimizer;
mod processor;
mod redis;
