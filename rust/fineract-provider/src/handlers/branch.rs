//! Office / branch HTTP handlers.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_branch::BranchRepository;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_command::CommandRequest;
use fineract_core::ids::OfficeId;

#[derive(Debug, Deserialize)]
pub struct CreateOfficeRequest {
    pub name: String,
    pub currency: String,
    pub external_id: Option<String>,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct OfficeResponse {
    pub id: i64,
    pub name: String,
    pub currency: String,
}

/// GET /v1/offices — list offices for the tenant.
pub async fn list_offices(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<OfficeResponse>>, AppError> {
    let ctx = &auth.0;
    let offices = state
        .office_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        offices
            .into_iter()
            .map(|o| OfficeResponse {
                id: o.id.value(),
                name: o.name,
                currency: o.currency.code().to_string(),
            })
            .collect(),
    ))
}

/// GET /v1/offices/:id — get a single office.
pub async fn get_office(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<OfficeResponse>, AppError> {
    let ctx = &auth.0;
    let office = state
        .office_repository
        .find_by_id(ctx, OfficeId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match office {
        Some(o) => Ok(Json(OfficeResponse {
            id: o.id.value(),
            name: o.name,
            currency: o.currency.code().to_string(),
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/offices — create a new office via the command pipeline.
pub async fn create_office(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateOfficeRequest>,
) -> Result<Json<OfficeResponse>, AppError> {
    let ctx = &auth.0;
    let currency = fineract_core::money::Currency::from_code(&payload.currency)
        .map_err(|_| AppError::BadRequest("invalid currency code".to_string()))?;

    let name = payload.name.clone();
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_branch::CreateOfficeCommand {
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            external_id: payload.external_id,
            opening_date: None,
            timezone: None,
            country: None,
            currency: currency.code().to_string(),
            parent_office_id: None,
            active: payload.active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let office_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(OfficeResponse {
                id: office_id.value(),
                name,
                currency: payload.currency,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/offices/:id — update an office.
pub async fn update_office(
    Extension(_auth): Extension<AuthContext>,
    _state: State<AppState>,
    Path(_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    Err(AppError::NotFound)
}

/// DELETE /v1/offices/:id — delete an office.
pub async fn delete_office(
    Extension(_auth): Extension<AuthContext>,
    _state: State<AppState>,
    Path(_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    Err(AppError::NotFound)
}
