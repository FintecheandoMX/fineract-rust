//! Command audit logging.
//!
//! Every command execution is recorded for traceability and compliance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::core::command::Command;
use crate::core::request::CommandRequest;
use crate::core::result::CommandProcessingResult;
use fineract_core::ids::{CommandId, TenantId};

/// A single audit record for a command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAuditRecord {
    pub command_id: CommandId,
    pub command_type: String,
    pub tenant_id: TenantId,
    pub initiated_by: Option<String>,
    pub executed_by: Option<String>,
    pub state: crate::core::state::CommandState,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// In-memory audit log. Production implementations should write to a
/// persistent store (e.g. database table or append-only log).
pub struct CommandAuditLog {
    records: Mutex<Vec<CommandAuditRecord>>,
}

impl CommandAuditLog {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }

    /// Record a command execution outcome.
    pub fn record<R: CommandRequest>(
        &self,
        command: &Command<R>,
        result: &CommandProcessingResult<R>,
    ) {
        let record = CommandAuditRecord {
            command_id: command.command_id,
            command_type: R::COMMAND_TYPE.to_string(),
            tenant_id: command.tenant_id,
            initiated_by: command.initiated_by.clone(),
            executed_by: result.executed_at.map(|_| "system".to_string()),
            state: result.state,
            created_at: command.created_at,
            executed_at: result.executed_at,
            error: result.error.clone(),
        };
        self.records.lock().unwrap().push(record);
    }

    /// Returns the number of recorded audit entries.
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.lock().unwrap().is_empty()
    }
}

impl Default for CommandAuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CommandAuditLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandAuditLog")
            .field("records", &self.records.lock().unwrap().len())
            .finish()
    }
}
