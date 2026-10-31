//! Command wrapper — carries idempotency key, audit timestamps, and payload.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::core::request::CommandRequest;
use crate::core::state::CommandState;
use fineract_core::ids::{CommandId, TenantId};

/// A command ready for (or already) dispatched through the pipeline.
///
/// Carries the idempotency key (client-supplied, used to detect duplicates),
/// audit timestamps, and the current lifecycle state.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct Command<R: CommandRequest> {
    pub command_id: CommandId,
    pub idempotency_key: Option<String>,
    pub tenant_id: TenantId,
    pub payload: R,
    pub state: CommandState,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub initiated_by: Option<String>,
    pub executed_by: Option<String>,
    pub approved_by: Option<String>,
    pub rejected_by: Option<String>,
    pub error: Option<String>,
}

impl<R: CommandRequest> Command<R> {
    /// Create a new command in Pending state.
    pub fn new(
        idempotency_key: Option<String>,
        tenant_id: TenantId,
        initiated_by: Option<String>,
        payload: R,
    ) -> Self {
        Self {
            command_id: CommandId::new(1), // placeholder; assigned by dispatcher/store
            idempotency_key,
            tenant_id,
            payload,
            state: CommandState::Pending,
            created_at: Utc::now(),
            updated_at: None,
            executed_at: None,
            approved_at: None,
            rejected_at: None,
            initiated_by,
            executed_by: None,
            approved_by: None,
            rejected_by: None,
            error: None,
        }
    }

    /// Returns true if this command is idempotent (has an idempotency key).
    pub fn is_idempotent(&self) -> bool {
        self.idempotency_key.is_some()
    }
}
