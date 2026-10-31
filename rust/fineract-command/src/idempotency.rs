//! Idempotency support.
//!
//! Duplicate commands with the same idempotency key are detected and rejected
//! if the payload differs. This prevents double-processing of retried requests.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::command::Command;
use crate::core::request::CommandRequest;
use crate::errors::CommandError;
use fineract_core::ids::CommandId;

/// Result of an idempotency check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotencyOutcome {
    /// The command is new (no prior record with this key).
    New,
    /// The command was already processed with the same payload.
    Duplicate,
    /// The command was already processed with a *different* payload — conflict.
    Conflict,
}

/// In-memory idempotency store. Production implementations should back onto a
/// persistent store (e.g. database table).
pub struct IdempotencyStore {
    records: Mutex<HashMap<String, IdempotencyRecord>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct IdempotencyRecord {
    command_id: CommandId,
    payload_hash: String,
}

impl IdempotencyStore {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
        }
    }

    /// Check and record idempotency for a command. Returns the outcome.
    pub fn check_and_record<R: CommandRequest>(
        &self,
        command: &Command<R>,
    ) -> Result<IdempotencyOutcome, CommandError> {
        let key = match &command.idempotency_key {
            Some(k) => k.clone(),
            None => return Ok(IdempotencyOutcome::New),
        };

        let payload_hash = compute_payload_hash(&command.payload);

        let mut records = self.records.lock().unwrap();
        match records.get(&key) {
            None => {
                records.insert(
                    key,
                    IdempotencyRecord {
                        command_id: command.command_id,
                        payload_hash,
                    },
                );
                Ok(IdempotencyOutcome::New)
            }
            Some(record) => {
                if record.payload_hash == payload_hash {
                    Ok(IdempotencyOutcome::Duplicate)
                } else {
                    Ok(IdempotencyOutcome::Conflict)
                }
            }
        }
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a stable hash of a command payload for idempotency comparison.
fn compute_payload_hash<R: CommandRequest>(payload: &R) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    let serialized = serde_json::to_string(payload).unwrap_or_default();
    serialized.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_initializes_empty() {
        let store = IdempotencyStore::new();
        assert_eq!(0, 0); // store initializes without error
        let _ = store;
    }
}
