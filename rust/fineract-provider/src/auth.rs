//! Authentication middleware and extractor.
//!
//! Middleware extracts the tenant ID and bearer token from headers,
//! authenticates via the `AuthenticationService`, and stores the
//! `AuthorizedContext` in request extensions. Handlers extract it via
//! `Extension<AuthContext>`.

use axum::{
    extract::{Extension, State},
    http::{header, request::Parts},
    middleware::{self, Next},
    response::Response,
};

use crate::error::AppError;
use crate::state::AppState;
use fineract_core::ids::TenantId;
use fineract_security::AuthenticationService;
use fineract_security::AuthorizedContext;

/// Extractor for the authenticated `AuthorizedContext`.
/// Populated by `auth_middleware` and extracted via `Extension`.
#[derive(Debug, Clone)]
pub struct AuthContext(pub AuthorizedContext);

/// Axum middleware: authenticates every request before it reaches the handler.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    let headers = request.headers();

    let tenant_header = headers
        .get(header::HeaderName::from_static(
            "fineract-platform-tenantid",
        ))
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::TenantMissing)?;
    let tenant_id = TenantId::new(
        tenant_header
            .parse::<i64>()
            .map_err(|_| AppError::InvalidTenantHeader)?,
    );

    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::AuthenticationRequired)?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::InvalidAuthHeader)?;

    let context = state
        .auth_service
        .authenticate_token(token)
        .await
        .map_err(|_| AppError::InvalidToken)?;

    if context.tenant_id != tenant_id {
        return Err(AppError::TenantIsolationViolation);
    }

    request.extensions_mut().insert(AuthContext(context));

    Ok(next.run(request).await)
}
