//! Core command pipeline traits and types.

pub mod command;
pub mod dispatcher;
pub mod handler;
pub mod hooks;
pub mod manager;
pub mod request;
pub mod result;
pub mod state;

pub use command::Command;
pub use dispatcher::CommandDispatcher;
pub use handler::CommandHandler;
pub use hooks::*;
pub use manager::CommandHandlerManager;
pub use request::CommandRequest;
pub use result::CommandProcessingResult;
pub use state::CommandState;
