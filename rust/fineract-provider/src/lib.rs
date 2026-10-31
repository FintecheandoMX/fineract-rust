//! REST API provider (Axum) — Phase 5.
//!
//! Wires HTTP routes to the command pipeline and domain services, with
//! authN/authZ on every route, tenant isolation from the
//! `Fineract-Platform-TenantId` header, and `deny_unknown_fields` on all
//! JSON request bodies.

#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

pub mod auth;
mod error;
pub mod handlers;
pub mod router;
pub mod state;

pub use auth::AuthContext;
pub use error::AppError;
pub use router::router;
pub use state::AppState;
