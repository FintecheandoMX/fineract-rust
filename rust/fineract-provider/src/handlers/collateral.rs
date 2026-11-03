//! Collateral HTTP handlers.
//! Collateral management for loans.

use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_collateral::Collateral;
use fineract_collateral::CollateralRepository;
use fineract_collateral::CollateralType;
use fineract_collateral::CreateCollateralCommand;
use fineract_collateral::DeleteCollateralCommand;
use fineract_collateral::UpdateCollateralCommand;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::CollateralId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;

#[derive(Debug, Deserialize)]
pub struct CreateCollateralRequest {
    pub loan_id: i64,
    pub collateral_type: String,
    pub description: Option<String>,
    pub value: String,
    pub owned_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollateralRequest {
    pub loan_id: i64,
    pub collateral_type: String,
    pub description: Option<String>,
    pub value: String,
    pub owned_by: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CollateralQueryParams {
    pub loan_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CollateralResponse {
    pub id: i64,
    pub loan_id: i64,
    pub collateral_type: String,
    pub description: Option<String>,
    pub value: String,
    pub owned_by: Option<String>,
    pub active: bool,
}

fn collateral_type_to_string(ct: CollateralType) -> String {
    match ct {
        CollateralType::Property => "property".to_string(),
        CollateralType::Vehicle => "vehicle".to_string(),
        CollateralType::Equipment => "equipment".to_string(),
        CollateralType::Inventory => "inventory".to_string(),
        CollateralType::Receivables => "receivables".to_string(),
        CollateralType::CashDeposit => "cash_deposit".to_string(),
        CollateralType::Guarantor => "guarantor".to_string(),
        CollateralType::Other => "other".to_string(),
    }
}

/// GET /v1/collateral — list collateral for the tenant, optionally filtered by loan_id.
pub async fn list_collateral(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<CollateralQueryParams>,
) -> Result<Json<Vec<CollateralResponse>>, AppError> {
    let ctx = &auth.0;

    let collateral = if let Some(loan_id) = params.loan_id {
        state
            .collateral_repository
            .find_by_loan_id(ctx, LoanId::new(loan_id))
            .await
            .map_err(|_| AppError::Internal)?
    } else {
        state
            .collateral_repository
            .find_all_by_tenant(ctx)
            .await
            .map_err(|_| AppError::Internal)?
    };

    Ok(Json(
        collateral
            .into_iter()
            .map(|c| CollateralResponse {
                id: c.id.value(),
                loan_id: c.loan_id.value(),
                collateral_type: collateral_type_to_string(c.collateral_type),
                description: c.description,
                value: c.value,
                owned_by: c.owned_by,
                active: c.active,
            })
            .collect(),
    ))
}

/// GET /v1/collateral/:id — get a single collateral.
pub async fn get_collateral(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CollateralResponse>, AppError> {
    let ctx = &auth.0;
    let collateral = state
        .collateral_repository
        .find_by_id(ctx, CollateralId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match collateral {
        Some(c) => Ok(Json(CollateralResponse {
            id: c.id.value(),
            loan_id: c.loan_id.value(),
            collateral_type: collateral_type_to_string(c.collateral_type),
            description: c.description,
            value: c.value,
            owned_by: c.owned_by,
            active: c.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/collateral — create a collateral via the command pipeline.
pub async fn create_collateral(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateCollateralRequest>,
) -> Result<Json<CollateralResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_collateral::CreateCollateralCommand {
            tenant_id: ctx.tenant_id,
            loan_id: LoanId::new(payload.loan_id),
            collateral_type: payload.collateral_type,
            description: payload.description,
            value: payload.value,
            owned_by: payload.owned_by,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let collateral_id = result.response.ok_or(AppError::Internal)?;
            let collateral = state
                .collateral_repository
                .find_by_id(ctx, collateral_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CollateralResponse {
                id: collateral.id.value(),
                loan_id: collateral.loan_id.value(),
                collateral_type: collateral_type_to_string(collateral.collateral_type),
                description: collateral.description,
                value: collateral.value,
                owned_by: collateral.owned_by,
                active: collateral.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/collateral/:id — update a collateral.
pub async fn update_collateral(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCollateralRequest>,
) -> Result<Json<CollateralResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_collateral::UpdateCollateralCommand {
            id: CollateralId::new(id),
            tenant_id: ctx.tenant_id,
            loan_id: LoanId::new(payload.loan_id),
            collateral_type: payload.collateral_type,
            description: payload.description,
            value: payload.value,
            owned_by: payload.owned_by,
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
            let collateral_id = result.response.ok_or(AppError::Internal)?;
            let collateral = state
                .collateral_repository
                .find_by_id(ctx, collateral_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CollateralResponse {
                id: collateral.id.value(),
                loan_id: collateral.loan_id.value(),
                collateral_type: collateral_type_to_string(collateral.collateral_type),
                description: collateral.description,
                value: collateral.value,
                owned_by: collateral.owned_by,
                active: collateral.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/collateral/:id — delete a collateral.
pub async fn delete_collateral(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_collateral::DeleteCollateralCommand {
            id: CollateralId::new(id),
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
