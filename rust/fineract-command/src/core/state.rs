//! Command lifecycle states.

use serde::{Deserialize, Serialize};

/// Lifecycle state of a command / maker-checker request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandState {
    /// Command received, awaiting approval (if maker-checker required).
    Pending,
    /// Command approved by a maker, ready for execution.
    Approved,
    /// Command rejected.
    Rejected,
    /// Command executed successfully.
    Executed,
    /// Command expired before execution.
    Expired,
}
