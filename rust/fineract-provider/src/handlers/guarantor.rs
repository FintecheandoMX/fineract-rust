//! Guarantor HTTP handlers.
//! Guarantor management for loans.

use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::GuarantorId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_guarantor::CreateGuarantorCommand;
use fineract_guarantor::DeleteGuarantorCommand;
use fineract_guarantor::Guarantor;
use fineract_guarantor::GuarantorRepository;
use fineract_guarantor::UpdateGuarantorCommand;

#[derive(Debug, Deserialize)]
pub struct CreateGuarantorRequest {
    pub loan_id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub identification_number: Option<String>,
    pub net_worth: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGuarantorRequest {
    pub loan_id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub identification_number: Option<String>,
    pub net_worth: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct GuarantorQueryParams {
    pub loan_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GuarantorResponse {
    pub id: i64,
    pub loan_id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub identification_number: Option<String>,
    pub net_worth: Option<String>,
    pub active: bool,
}

/// GET /v1/guarantors — list guarantors for the tenant, optionally filtered by loan_id.
pub async fn list_guarantors(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<GuarantorQueryParams>,
) -> Result<Json<Vec<GuarantorResponse>>, AppError> {
    let ctx = &auth.0;

    let guarantors = if let Some(loan_id) = params.loan_id {
        state
            .guarantor_repository
            .find_by_loan_id(ctx, LoanId::new(loan_id))
            .await
            .map_err(|_| AppError::Internal)?
    } else {
        state
            .guarantor_repository
            .find_all_by_tenant(ctx)
            .await
            .map_err(|_| AppError::Internal)?
    };

    Ok(Json(
        guarantors
            .into_iter()
            .map(|g| GuarantorResponse {
                id: g.id.value(),
                loan_id: g.loan_id.value(),
                name: g.name,
                email: g.email,
                phone: g.phone,
                address: g.address,
                identification_number: g.identification_number,
                net_worth: g.net_worth,
                active: g.active,
            })
            .collect(),
    ))
}

/// GET /v1/guarantors/:id — get a single guarantor.
pub async fn get_guarantor(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<GuarantorResponse>, AppError> {
    let ctx = &auth.0;
    let guarantor = state
        .guarantor_repository
        .find_by_id(ctx, GuarantorId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match guarantor {
        Some(g) => Ok(Json(GuarantorResponse {
            id: g.id.value(),
            loan_id: g.loan_id.value(),
            name: g.name,
            email: g.email,
            phone: g.phone,
            address: g.address,
            identification_number: g.identification_number,
            net_worth: g.net_worth,
            active: g.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/guarantors — create a guarantor via the command pipeline.
pub async fn create_guarantor(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateGuarantorRequest>,
) -> Result<Json<GuarantorResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_guarantor::CreateGuarantorCommand {
            tenant_id: ctx.tenant_id,
            loan_id: LoanId::new(payload.loan_id),
            name: payload.name,
            email: payload.email,
            phone: payload.phone,
            address: payload.address,
            identification_number: payload.identification_number,
            net_worth: payload.net_worth,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let guarantor_id = result.response.ok_or(AppError::Internal)?;
            let guarantor = state
                .guarantor_repository
                .find_by_id(ctx, guarantor_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(GuarantorResponse {
                id: guarantor.id.value(),
                loan_id: guarantor.loan_id.value(),
                name: guarantor.name,
                email: guarantor.email,
                phone: guarantor.phone,
                address: guarantor.address,
                identification_number: guarantor.identification_number,
                net_worth: guarantor.net_worth,
                active: guarantor.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/guarantors/:id — update a guarantor.
pub async fn update_guarantor(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateGuarantorRequest>,
) -> Result<Json<GuarantorResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_guarantor::UpdateGuarantorCommand {
            id: GuarantorId::new(id),
            tenant_id: ctx.tenant_id,
            loan_id: LoanId::new(payload.loan_id),
            name: payload.name,
            email: payload.email,
            phone: payload.phone,
            address: payload.address,
            identification_number: payload.identification_number,
            net_worth: payload.net_worth,
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
            let guarantor_id = result.response.ok_or(AppError::Internal)?;
            let guarantor = state
                .guarantor_repository
                .find_by_id(ctx, guarantor_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(GuarantorResponse {
                id: guarantor.id.value(),
                loan_id: guarantor.loan_id.value(),
                name: guarantor.name,
                email: guarantor.email,
                phone: guarantor.phone,
                address: guarantor.address,
                identification_number: guarantor.identification_number,
                net_worth: guarantor.net_worth,
                active: guarantor.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/guarantors/:id — delete a guarantor.
pub async fn delete_guarantor(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_guarantor::DeleteGuarantorCommand {
            id: GuarantorId::new(id),
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
