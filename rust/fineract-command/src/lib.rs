//! Command pipeline (CQRS) for Fineract.
//!
//! Phase 3 implements:
//! - [`CommandRequest`] trait — typed command payloads (never raw JsonCommand)
//! - [`Command`] — command wrapper with idempotency key and audit timestamps
//! - [`CommandHandler`] trait — handles a specific command type with authZ
//! - [`CommandDispatcher`] trait — sync + async dispatch to registered handlers
//! - [`CommandHandlerManager`] — registry of command handlers
//! - Idempotency support — duplicate command detection
//! - Maker-checker support — pending/approved/rejected/executed lifecycle
//! - Audit logging — every command execution is recorded
//!
//! Every command handler receives an explicit [`AuthorizedContext`] and must
//! call [`AuthorizationService::check`] before mutating state. This makes
//! authorization a compile-time requirement for all writes.

#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

mod audit;
mod core;
mod errors;
mod idempotency;
mod implementation;
mod maker_checker;

pub use audit::*;
pub use core::*;
pub use errors::*;
pub use idempotency::*;
pub use implementation::dispatcher::{AsyncCommandDispatcher, SynchronousCommandDispatcher};
pub use maker_checker::*;
