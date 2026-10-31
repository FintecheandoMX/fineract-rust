//! Command pipeline errors.

use thiserror::Error;

/// Error produced by the command pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommandError {
    #[error("command handler not found for type: {0}")]
    HandlerNotFound(String),

    #[error("command handler returned an error: {0}")]
    HandlerError(String),

    #[error("idempotency key already processed with a different payload")]
    IdempotencyConflict,

    #[error("command is in invalid state: {0}")]
    InvalidState(String),

    #[error("authorization denied: {0}")]
    Unauthorized(String),

    #[error("internal command pipeline error: {0}")]
    Internal(String),

    #[error("maker-checker approval required")]
    ApprovalRequired,

    #[error("command not found: {0}")]
    NotFound(fineract_core::ids::CommandId),
}
