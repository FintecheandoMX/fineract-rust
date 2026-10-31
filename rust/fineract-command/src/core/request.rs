//! Command request trait — typed payload for a state-changing operation.
//!
//! Every command payload implements this trait. It carries its own command type
//! name (for dispatch) and response type. Payloads are serde-serializable for
//! audit logging and idempotency storage.

use serde::Serialize;

/// A request payload for a command. Implementors are strongly typed domain
/// requests (e.g. `CreateLoanCommand`, `ApproveLoanCommand`) — never raw
/// `JsonCommand` or `serde_json::Value`.
pub trait CommandRequest:
    Send + Sync + 'static + Serialize + serde::de::DeserializeOwned + std::fmt::Debug + Clone
{
    /// The response type produced by handling this command.
    type Response: Send
        + Sync
        + 'static
        + Serialize
        + serde::de::DeserializeOwned
        + std::fmt::Debug
        + Clone;

    /// Unique command type name, used for handler dispatch.
    const COMMAND_TYPE: &'static str;
}
