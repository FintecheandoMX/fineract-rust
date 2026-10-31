//! Charge HTTP handlers.
//! Charges (fees) management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_charge::Charge;
use fineract_charge::ChargeRepository;
use fineract_charge::ChargeType;
use fineract_charge::CreateChargeCommand;
use fineract_charge::DeleteChargeCommand;
use fineract_charge::UpdateChargeCommand;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::ChargeId;
use fineract_core::ids::TenantId;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateChargeRequest {
    pub name: String,
    pub charge_type: String,
    pub value: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChargeRequest {
    pub name: String,
    pub charge_type: String,
    pub value: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct ChargeResponse {
    pub id: i64,
    pub name: String,
    pub charge_type: String,
    pub value: String,
    pub active: bool,
}

/// GET /v1/charges — list charges for the tenant.
pub async fn list_charges(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<ChargeResponse>>, AppError> {
    let ctx = &auth.0;
    let charges = state
        .charge_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        charges
            .into_iter()
            .map(|c| ChargeResponse {
                id: c.id.value(),
                name: c.name,
                charge_type: format!("{:?}", c.charge_type),
                value: c.value.to_string(),
                active: c.active,
            })
            .collect(),
    ))
}

/// GET /v1/charges/:id — get a single charge.
pub async fn get_charge(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ChargeResponse>, AppError> {
    let ctx = &auth.0;
    let charge = state
        .charge_repository
        .find_by_id(ctx, ChargeId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match charge {
        Some(c) => Ok(Json(ChargeResponse {
            id: c.id.value(),
            name: c.name,
            charge_type: format!("{:?}", c.charge_type),
            value: c.value.to_string(),
            active: c.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/charges — create a charge via the command pipeline.
pub async fn create_charge(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateChargeRequest>,
) -> Result<Json<ChargeResponse>, AppError> {
    let ctx = &auth.0;
    let charge_type = match payload.charge_type.as_str() {
        "Fixed" => ChargeType::Fixed,
        "Percentage" => ChargeType::Percentage,
        _ => return Err(AppError::BadRequest("invalid charge type".to_string())),
    };
    let value = Decimal::from_str_exact(&payload.value)
        .map_err(|_| AppError::BadRequest("invalid value".to_string()))?;
    let name = payload.name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_charge::CreateChargeCommand {
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            charge_type: format!("{:?}", charge_type),
            value: value.to_string(),
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
            let charge_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(ChargeResponse {
                id: charge_id.value(),
                name,
                charge_type: payload.charge_type,
                value: value.to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/charges/:id — update a charge.
pub async fn update_charge(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateChargeRequest>,
) -> Result<Json<ChargeResponse>, AppError> {
    let ctx = &auth.0;
    let charge_type = match payload.charge_type.as_str() {
        "Fixed" => ChargeType::Fixed,
        "Percentage" => ChargeType::Percentage,
        _ => return Err(AppError::BadRequest("invalid charge type".to_string())),
    };
    let value = Decimal::from_str_exact(&payload.value)
        .map_err(|_| AppError::BadRequest("invalid value".to_string()))?;
    let name = payload.name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_charge::UpdateChargeCommand {
            id: ChargeId::new(id),
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            charge_type: format!("{:?}", charge_type),
            value: value.to_string(),
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
            let charge_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(ChargeResponse {
                id: charge_id.value(),
                name,
                charge_type: payload.charge_type,
                value: value.to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/charges/:id — delete a charge.
pub async fn delete_charge(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_charge::DeleteChargeCommand {
            id: ChargeId::new(id),
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
