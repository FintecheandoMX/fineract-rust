//! Command processing result.

use serde::Serialize;

/// Outcome of dispatching a command through the pipeline.
#[derive(Debug, Clone, Serialize)]
pub struct CommandProcessingResult<R: crate::core::request::CommandRequest> {
    pub command_id: fineract_core::ids::CommandId,
    pub state: crate::core::state::CommandState,
    pub response: Option<R::Response>,
    pub error: Option<String>,
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
}
