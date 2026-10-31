//! Tax HTTP handlers.
//! Tax components management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::TaxId;
use fineract_core::ids::TenantId;
use fineract_tax::CreateTaxCommand;
use fineract_tax::DeleteTaxCommand;
use fineract_tax::Tax;
use fineract_tax::TaxInclusion;
use fineract_tax::TaxRepository;
use fineract_tax::UpdateTaxCommand;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateTaxRequest {
    pub name: String,
    pub rate: String,
    pub inclusion: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaxRequest {
    pub name: String,
    pub rate: String,
    pub inclusion: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct TaxResponse {
    pub id: i64,
    pub name: String,
    pub rate: String,
    pub inclusion: String,
    pub active: bool,
}

/// GET /v1/taxes — list taxes for the tenant.
pub async fn list_taxes(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<TaxResponse>>, AppError> {
    let ctx = &auth.0;
    let taxes = state
        .tax_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        taxes
            .into_iter()
            .map(|t| TaxResponse {
                id: t.id.value(),
                name: t.name,
                rate: t.rate.to_string(),
                inclusion: format!("{:?}", t.inclusion),
                active: t.active,
            })
            .collect(),
    ))
}

/// GET /v1/taxes/:id — get a single tax.
pub async fn get_tax(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<TaxResponse>, AppError> {
    let ctx = &auth.0;
    let tax = state
        .tax_repository
        .find_by_id(ctx, TaxId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match tax {
        Some(t) => Ok(Json(TaxResponse {
            id: t.id.value(),
            name: t.name,
            rate: t.rate.to_string(),
            inclusion: format!("{:?}", t.inclusion),
            active: t.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/taxes — create a tax via the command pipeline.
pub async fn create_tax(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateTaxRequest>,
) -> Result<Json<TaxResponse>, AppError> {
    let ctx = &auth.0;
    let inclusion = match payload.inclusion.as_str() {
        "Included" => TaxInclusion::Included,
        "Excluded" => TaxInclusion::Excluded,
        _ => return Err(AppError::BadRequest("invalid inclusion".to_string())),
    };
    let rate = Decimal::from_str_exact(&payload.rate)
        .map_err(|_| AppError::BadRequest("invalid rate".to_string()))?;
    let name = payload.name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_tax::CreateTaxCommand {
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            rate: rate.to_string(),
            inclusion: format!("{:?}", inclusion),
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
            let tax_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(TaxResponse {
                id: tax_id.value(),
                name,
                rate: rate.to_string(),
                inclusion: payload.inclusion,
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/taxes/:id — update a tax.
pub async fn update_tax(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTaxRequest>,
) -> Result<Json<TaxResponse>, AppError> {
    let ctx = &auth.0;
    let inclusion = match payload.inclusion.as_str() {
        "Included" => TaxInclusion::Included,
        "Excluded" => TaxInclusion::Excluded,
        _ => return Err(AppError::BadRequest("invalid inclusion".to_string())),
    };
    let rate = Decimal::from_str_exact(&payload.rate)
        .map_err(|_| AppError::BadRequest("invalid rate".to_string()))?;
    let name = payload.name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_tax::UpdateTaxCommand {
            id: TaxId::new(id),
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            rate: rate.to_string(),
            inclusion: format!("{:?}", inclusion),
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
            let tax_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(TaxResponse {
                id: tax_id.value(),
                name,
                rate: rate.to_string(),
                inclusion: payload.inclusion,
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/taxes/:id — delete a tax.
pub async fn delete_tax(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_tax::DeleteTaxCommand {
            id: TaxId::new(id),
            tenant_id: ctx.tenant_id,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => Ok(StatusCode::NO_CONTENT),
        _ => Err(AppError::Internal),
    }
}
