//! Application error type and mapping to HTTP responses.
//!
//! Domain and security errors are mapped to HTTP status codes without
//! leaking internal details (no SQL, stack traces, or file paths).

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde_json::json;

/// Top-level application error. Every variant maps to a safe HTTP response.
#[derive(Debug)]
pub enum AppError {
    /// Authentication is required but no credentials were provided.
    AuthenticationRequired,
    /// The Authorization header was malformed.
    InvalidAuthHeader,
    /// The bearer token was invalid or expired.
    InvalidToken,
    /// The Fineract-Platform-TenantId header was missing.
    TenantMissing,
    /// The tenant header could not be parsed as an i64.
    InvalidTenantHeader,
    /// The authenticated tenant does not match the requested tenant.
    TenantIsolationViolation,
    /// The caller lacks the required permission.
    InsufficientPermissions,
    /// A requested resource was not found.
    NotFound,
    /// A validation or business rule was violated.
    BadRequest(String),
    /// An internal error occurred (mapped to 500 with no details leaked).
    Internal,
}

impl From<fineract_security::SecurityError> for AppError {
    fn from(e: fineract_security::SecurityError) -> Self {
        match e {
            fineract_security::SecurityError::AuthenticationRequired => {
                AppError::AuthenticationRequired
            }
            fineract_security::SecurityError::InvalidCredentials
            | fineract_security::SecurityError::InvalidToken
            | fineract_security::SecurityError::TokenExpired
            | fineract_security::SecurityError::TokenRevoked => AppError::InvalidToken,
            fineract_security::SecurityError::InsufficientPermissions { .. } => {
                AppError::InsufficientPermissions
            }
            fineract_security::SecurityError::TenantMissing => AppError::TenantMissing,
            fineract_security::SecurityError::TenantIsolationViolation => {
                AppError::TenantIsolationViolation
            }
            fineract_security::SecurityError::AccessDenied { .. } => {
                AppError::InsufficientPermissions
            }
            fineract_security::SecurityError::AccountLocked
            | fineract_security::SecurityError::TwoFactorRequired
            | fineract_security::SecurityError::InvalidTwoFactorCode => {
                AppError::BadRequest("authentication challenge required".to_string())
            }
            fineract_security::SecurityError::Internal { .. } => AppError::Internal,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::AuthenticationRequired => (
                StatusCode::UNAUTHORIZED,
                "authentication_required",
                "authentication is required",
            ),
            AppError::InvalidAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "invalid_auth_header",
                "use Authorization: Bearer <token>",
            ),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "the provided token is invalid or expired",
            ),
            AppError::TenantMissing => (
                StatusCode::BAD_REQUEST,
                "tenant_missing",
                "Fineract-Platform-TenantId header is required",
            ),
            AppError::InvalidTenantHeader => (
                StatusCode::BAD_REQUEST,
                "invalid_tenant_header",
                "tenant ID must be a numeric integer",
            ),
            AppError::TenantIsolationViolation => (
                StatusCode::FORBIDDEN,
                "tenant_isolation_violation",
                "the requested resource belongs to a different tenant",
            ),
            AppError::InsufficientPermissions => (
                StatusCode::FORBIDDEN,
                "insufficient_permissions",
                "the caller does not have the required permission",
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "the requested resource was not found",
            ),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.as_str()),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "an internal error occurred",
            ),
        };

        let body = Json(json!({
            "error": error_code,
            "message": message,
        }));

        (status, body).into_response()
    }
}
