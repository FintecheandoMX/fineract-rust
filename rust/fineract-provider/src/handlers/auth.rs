//! Authentication handlers (login, refresh, logout).

use axum::extract::State;
use axum::Json;
use http::StatusCode;
use secrecy::SecretString;
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

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub revoked: bool,
}

/// POST /auth/login — authenticate with username/password, issue JWT pair.
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let password = SecretString::from(payload.password);
    let _context = state
        .auth_service
        .authenticate_password(
            fineract_core::ids::TenantId::new(1),
            &payload.username,
            &password,
        )
        .await
        .map_err(|_| AppError::InvalidToken)?;

    let ttl = payload.ttl_seconds.unwrap_or(900);
    let _pair = state
        .auth_service
        .issue_token_pair(
            &fineract_security::AuthorizedContext::new(
                fineract_core::ids::TenantId::new(1),
                fineract_security::Principal {
                    app_user_id: fineract_core::ids::AppUserId::new(1),
                    username: payload.username,
                    roles: Default::default(),
                    permissions: Default::default(),
                    office_id: None,
                },
                fineract_security::SessionId::new(),
                chrono::Utc::now(),
            ),
            ttl,
        )
        .await
        .map_err(|_| AppError::Internal)?;

    Err(AppError::Internal)
}

/// POST /auth/refresh — exchange a refresh token for a new access token.
pub async fn refresh_token(State(_state): State<AppState>) -> Result<StatusCode, AppError> {
    Err(AppError::Internal)
}

/// POST /auth/logout — revoke the refresh token.
pub async fn logout(
    State(_state): State<AppState>,
) -> Result<axum::Json<LogoutResponse>, AppError> {
    Err(AppError::Internal)
}

/// GET /healthz — liveness/readiness probe (no auth required).
pub async fn healthz() -> &'static str {
    "ok"
}
