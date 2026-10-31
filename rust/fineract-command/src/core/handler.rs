//! Command handler trait.
//!
//! Every command type has a handler that executes the state-changing operation.
//! Handlers receive an explicit [`AuthorizedContext`] and must call the
//! [`AuthorizationService`] before mutating data — authorization is mandatory.

use async_trait::async_trait;

use crate::core::request::CommandRequest;
use crate::errors::CommandError;
use fineract_security::AuthorizedContext;

/// Handles commands of type `R`.
///
/// Implementors must enforce authorization at the domain service layer, not
/// just the API layer. This is the primary IDOR defense for writes.
#[async_trait]
pub trait CommandHandler<R: CommandRequest>: Send + Sync {
    /// The command type this handler handles.
    const COMMAND_TYPE: &'static str = R::COMMAND_TYPE;

    /// Execute the command. Returns the response on success or a
    /// [`CommandError`] on failure (including authorization denial).
    async fn handle(
        &self,
        request: R,
        context: &AuthorizedContext,
    ) -> Result<R::Response, CommandError>;
}
