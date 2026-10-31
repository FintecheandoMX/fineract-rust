//! Command hooks — cross-cutting concerns executed around command handling.
//!
//! Hooks can add timestamps, usernames, or other metadata to commands before
//! and after execution.

use async_trait::async_trait;

use crate::core::command::Command;
use crate::core::request::CommandRequest;

/// A hook executed around command handling.
#[async_trait]
pub trait CommandHook: Send + Sync {
    /// Executed before the command is handled.
    async fn before<R: CommandRequest>(
        &self,
        command: &mut Command<R>,
    ) -> Result<(), crate::errors::CommandError>;

    /// Executed after the command is handled successfully.
    async fn after<R: CommandRequest>(
        &self,
        command: &Command<R>,
        result: &crate::core::result::CommandProcessingResult<R>,
    ) -> Result<(), crate::errors::CommandError>;

    /// Executed when the command handling fails.
    async fn error<R: CommandRequest>(
        &self,
        command: &Command<R>,
        error: &crate::errors::CommandError,
    ) -> Result<(), crate::errors::CommandError>;
}

/// Default no-op hook.
pub struct NoOpHook;

#[async_trait]
impl CommandHook for NoOpHook {
    async fn before<R: CommandRequest>(
        &self,
        _command: &mut Command<R>,
    ) -> Result<(), crate::errors::CommandError> {
        Ok(())
    }

    async fn after<R: CommandRequest>(
        &self,
        _command: &Command<R>,
        _result: &crate::core::result::CommandProcessingResult<R>,
    ) -> Result<(), crate::errors::CommandError> {
        Ok(())
    }

    async fn error<R: CommandRequest>(
        &self,
        _command: &Command<R>,
        _error: &crate::errors::CommandError,
    ) -> Result<(), crate::errors::CommandError> {
        Ok(())
    }
}

/// Timestamp hook — sets created_at and initiated_by on the command.
pub struct TimestampHook;

#[async_trait]
impl CommandHook for TimestampHook {
    async fn before<R: CommandRequest>(
        &self,
        command: &mut Command<R>,
    ) -> Result<(), crate::errors::CommandError> {
        command.created_at = chrono::Utc::now();
        command.state = crate::core::state::CommandState::Pending;
        Ok(())
    }

    async fn after<R: CommandRequest>(
        &self,
        _command: &Command<R>,
        _result: &crate::core::result::CommandProcessingResult<R>,
    ) -> Result<(), crate::errors::CommandError> {
        Ok(())
    }

    async fn error<R: CommandRequest>(
        &self,
        _command: &Command<R>,
        _error: &crate::errors::CommandError,
    ) -> Result<(), crate::errors::CommandError> {
        Ok(())
    }
}
