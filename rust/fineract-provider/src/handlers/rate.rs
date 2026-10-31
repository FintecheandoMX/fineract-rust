//! Rate HTTP handlers.
//! Interest rates and rate tiers management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::RateId;
use fineract_core::ids::TenantId;
use fineract_rates::CreateRateCommand;
use fineract_rates::DeleteRateCommand;
use fineract_rates::Rate;
use fineract_rates::RatePeriod;
use fineract_rates::RateRepository;
use fineract_rates::UpdateRateCommand;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateRateRequest {
    pub name: String,
    pub period: String,
    pub annual_percentage_rate: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRateRequest {
    pub name: String,
    pub period: String,
    pub annual_percentage_rate: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct RateResponse {
    pub id: i64,
    pub name: String,
    pub period: String,
    pub annual_percentage_rate: String,
    pub active: bool,
}

/// GET /v1/rates — list rates for the tenant.
pub async fn list_rates(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<RateResponse>>, AppError> {
    let ctx = &auth.0;
    let rates = state
        .rate_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        rates
            .into_iter()
            .map(|r| RateResponse {
                id: r.id.value(),
                name: r.name,
                period: format!("{:?}", r.period),
                annual_percentage_rate: r.annual_percentage_rate.to_string(),
                active: r.active,
            })
            .collect(),
    ))
}

/// GET /v1/rates/:id — get a single rate.
pub async fn get_rate(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<RateResponse>, AppError> {
    let ctx = &auth.0;
    let rate = state
        .rate_repository
        .find_by_id(ctx, RateId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match rate {
        Some(r) => Ok(Json(RateResponse {
            id: r.id.value(),
            name: r.name,
            period: format!("{:?}", r.period),
            annual_percentage_rate: r.annual_percentage_rate.to_string(),
            active: r.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/rates — create a rate via the command pipeline.
pub async fn create_rate(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateRateRequest>,
) -> Result<Json<RateResponse>, AppError> {
    let ctx = &auth.0;
    let period = match payload.period.as_str() {
        "Daily" => RatePeriod::Daily,
        "Weekly" => RatePeriod::Weekly,
        "Monthly" => RatePeriod::Monthly,
        "Yearly" => RatePeriod::Yearly,
        _ => return Err(AppError::BadRequest("invalid period".to_string())),
    };
    let apr = Decimal::from_str_exact(&payload.annual_percentage_rate)
        .map_err(|_| AppError::BadRequest("invalid annual_percentage_rate".to_string()))?;
    let name = payload.name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_rates::CreateRateCommand {
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            period: format!("{:?}", period),
            annual_percentage_rate: apr.to_string(),
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
            let rate_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(RateResponse {
                id: rate_id.value(),
                name,
                period: payload.period,
                annual_percentage_rate: apr.to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/rates/:id — update a rate.
pub async fn update_rate(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateRateRequest>,
) -> Result<Json<RateResponse>, AppError> {
    let ctx = &auth.0;
    let period = match payload.period.as_str() {
        "Daily" => RatePeriod::Daily,
        "Weekly" => RatePeriod::Weekly,
        "Monthly" => RatePeriod::Monthly,
        "Yearly" => RatePeriod::Yearly,
        _ => return Err(AppError::BadRequest("invalid period".to_string())),
    };
    let apr = Decimal::from_str_exact(&payload.annual_percentage_rate)
        .map_err(|_| AppError::BadRequest("invalid annual_percentage_rate".to_string()))?;
    let name = payload.name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_rates::UpdateRateCommand {
            id: RateId::new(id),
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            period: format!("{:?}", period),
            annual_percentage_rate: apr.to_string(),
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
            let rate_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(RateResponse {
                id: rate_id.value(),
                name,
                period: payload.period,
                annual_percentage_rate: apr.to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/rates/:id — delete a rate.
pub async fn delete_rate(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_rates::DeleteRateCommand {
            id: RateId::new(id),
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
