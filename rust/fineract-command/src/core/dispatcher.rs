//! Command dispatcher trait.
//!
//! Dispatches commands to their registered handlers. Both synchronous and
//! asynchronous implementations are provided.

use async_trait::async_trait;

use crate::core::command::Command;
use crate::core::request::CommandRequest;
use crate::core::result::CommandProcessingResult;
use crate::errors::CommandError;
use fineract_security::AuthorizedContext;

/// Dispatches a command to its registered handler.
#[async_trait]
pub trait CommandDispatcher: Send + Sync {
    /// Dispatch a command to its handler. Handles idempotency checks,
    /// maker-checker workflow, and audit logging.
    async fn dispatch<R: CommandRequest>(
        &self,
        command: Command<R>,
        context: &AuthorizedContext,
    ) -> Result<CommandProcessingResult<R>, CommandError>;
}
