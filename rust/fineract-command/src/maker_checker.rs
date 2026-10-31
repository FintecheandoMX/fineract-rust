//! Maker-checker support.
//!
//! Commands that require maker-checker follow the lifecycle:
//! Pending -> Approved -> Executed, or Pending -> Rejected.
//!
//! Commands that do not require maker-checker go directly Pending -> Executed.

use serde::{Deserialize, Serialize};

/// Whether a command requires maker-checker approval before execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MakerCheckerPolicy {
    /// Command executes immediately without separate approval.
    Direct,
    /// Command requires approval from a different user before execution.
    Required,
}

/// Result of a maker-checker decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MakerCheckerDecision {
    /// Command approved for execution.
    Approved,
    /// Command rejected.
    Rejected,
    /// Command still pending approval.
    Pending,
}
