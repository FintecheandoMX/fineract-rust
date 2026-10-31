//! Investor HTTP handlers.
//! Investor management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::InvestorId;
use fineract_core::ids::TenantId;
use fineract_investor::CreateInvestorCommand;
use fineract_investor::DeleteInvestorCommand;
use fineract_investor::Investor;
use fineract_investor::InvestorRepository;
use fineract_investor::UpdateInvestorCommand;

#[derive(Debug, Deserialize)]
pub struct CreateInvestorRequest {
    pub name: String,
    pub external_id: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInvestorRequest {
    pub name: String,
    pub external_id: Option<String>,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct InvestorResponse {
    pub id: i64,
    pub name: String,
    pub external_id: Option<String>,
    pub active: bool,
}

/// GET /v1/investors — list investors for the tenant.
pub async fn list_investors(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<InvestorResponse>>, AppError> {
    let ctx = &auth.0;
    let investors = state
        .investor_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        investors
            .into_iter()
            .map(|i| InvestorResponse {
                id: i.id.value(),
                name: i.name,
                external_id: i.external_id,
                active: i.active,
            })
            .collect(),
    ))
}

/// GET /v1/investors/:id — get a single investor.
pub async fn get_investor(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<InvestorResponse>, AppError> {
    let ctx = &auth.0;
    let investor = state
        .investor_repository
        .find_by_id(ctx, InvestorId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match investor {
        Some(i) => Ok(Json(InvestorResponse {
            id: i.id.value(),
            name: i.name,
            external_id: i.external_id,
            active: i.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/investors — create an investor via the command pipeline.
pub async fn create_investor(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateInvestorRequest>,
) -> Result<Json<InvestorResponse>, AppError> {
    let ctx = &auth.0;
    let name = payload.name.clone();
    let external_id = payload.external_id.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_investor::CreateInvestorCommand {
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            external_id: external_id.clone(),
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
            let investor_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(InvestorResponse {
                id: investor_id.value(),
                name,
                external_id,
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/investors/:id — update an investor.
pub async fn update_investor(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateInvestorRequest>,
) -> Result<Json<InvestorResponse>, AppError> {
    let ctx = &auth.0;
    let name = payload.name.clone();
    let external_id = payload.external_id.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_investor::UpdateInvestorCommand {
            id: InvestorId::new(id),
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            external_id: external_id.clone(),
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
            let investor_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(InvestorResponse {
                id: investor_id.value(),
                name,
                external_id,
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/investors/:id — delete an investor.
pub async fn delete_investor(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_investor::DeleteInvestorCommand {
            id: InvestorId::new(id),
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
