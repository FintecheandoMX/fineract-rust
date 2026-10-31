//! Security error types.
//!
//! All variants map to HTTP status codes at the API layer without leaking
//! internal details (no stack traces, SQL, or file paths).

use thiserror::Error;

/// Security-related error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SecurityError {
    #[error("authentication required")]
    AuthenticationRequired,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("account locked: too many failed attempts")]
    AccountLocked,

    #[error("token expired")]
    TokenExpired,

    #[error("invalid token")]
    InvalidToken,

    #[error("token revoked")]
    TokenRevoked,

    #[error("insufficient permissions: {permission}")]
    InsufficientPermissions { permission: String },

    #[error("access denied: {detail}")]
    AccessDenied { detail: String },

    #[error("tenant context missing")]
    TenantMissing,

    #[error("tenant isolation violation")]
    TenantIsolationViolation,

    #[error("two-factor authentication required")]
    TwoFactorRequired,

    #[error("invalid two-factor code")]
    InvalidTwoFactorCode,

    #[error("internal security error: {detail}")]
    Internal { detail: String },
}
