//! Authentication, authorization and multi-tenancy security layer.
//!
//! Phase 2 implements:
//! - [`AuthorizedContext`] — explicit, request-scoped security context carrying
//!   tenant + authenticated user + roles/permissions. Impossible to forget:
//!   every domain service takes it as an explicit parameter.
//! - Authentication: Basic Auth (argon2-hashed passwords) and JWT (access +
//!   refresh tokens, constant-time comparison, revocation).
//! - Authorization: RBAC + data-level checks via [`AuthorizationService`].
//!   IDOR protection by construction — every data access receives and checks
//!   `AuthorizedContext`.
//!
//! Security: compiled with `#![forbid(unsafe_code)]`. Secrets use `secrecy` /
//! `zeroize`; comparisons use `subtle` (constant time); randomness uses `rand`
//! CSPRNG.

#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

mod authentication;
mod authorization;
mod context;
mod errors;
mod password;
#[cfg(test)]
mod tests;
mod tokens;

pub use authentication::*;
pub use authorization::*;
pub use context::*;
pub use errors::*;
pub use password::*;
pub use tokens::*;
