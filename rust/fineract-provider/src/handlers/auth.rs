//! Authentication handlers (login, refresh, logout).

use axum::extract::State;
use axum::Json;
use http::StatusCode;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;
use fineract_security::AuthenticationService;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub ttl_seconds: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub revoked: bool,
}

/// POST /auth/login — authenticate with username/password, issue JWT pair.
///
/// Requires `Fineract-Platform-TenantId` header and JSON body with username/password.
pub async fn login(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Extract tenant from header
    let tenant_header = headers
        .get("Fineract-Platform-TenantId")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::TenantMissing)?;
    let tenant_id = fineract_core::ids::TenantId::new(
        tenant_header
            .parse::<i64>()
            .map_err(|_| AppError::InvalidTenantHeader)?,
    );

    let password = SecretString::from(payload.password);
    let context = state
        .auth_service
        .authenticate_password(tenant_id, &payload.username, &password)
        .await
        .map_err(|e| match e {
            fineract_security::SecurityError::InvalidCredentials => AppError::InvalidToken,
            fineract_security::SecurityError::AccessDenied { .. } => AppError::InvalidToken,
            _ => AppError::Internal,
        })?;

    let ttl = payload.ttl_seconds.unwrap_or(900);
    let pair = state
        .auth_service
        .issue_token_pair(&context, ttl)
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(Json(LoginResponse {
        access_token: pair.access_token.clone(),
        refresh_token: pair.refresh_token.expose_secret().clone(),
        expires_in: ttl,
    }))
}

/// POST /auth/refresh — exchange a refresh token for a new access token.
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let refresh_token = SecretString::from(payload.refresh_token);

    // Revoke the old refresh token
    state
        .auth_service
        .revoke_refresh_token(&refresh_token)
        .await
        .map_err(|_| AppError::Internal)?;

    // For simplicity, we validate the refresh token and issue a new pair.
    // In a full implementation, we'd decode the refresh token to get the principal.
    // For now, we'll require the user to re-login for a new pair.
    Err(AppError::BadRequest(
        "Refresh token flow requires full implementation with token decode".to_string(),
    ))
}

/// POST /auth/logout — revoke the refresh token.
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<axum::Json<LogoutResponse>, AppError> {
    let refresh_token = SecretString::from(payload.refresh_token);

    state
        .auth_service
        .revoke_refresh_token(&refresh_token)
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(axum::Json(LogoutResponse { revoked: true }))
}

/// GET /healthz — liveness/readiness probe (no auth required).
pub async fn healthz() -> &'static str {
    "ok"
}
