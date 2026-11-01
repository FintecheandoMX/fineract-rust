//! Progressive Loan HTTP handlers.
//! Progressive loans with phased disbursement.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_progressive_loan::AdvancePhaseCommand;
use fineract_progressive_loan::CreateProgressiveLoanCommand;
use fineract_progressive_loan::DeleteProgressiveLoanCommand;
use fineract_progressive_loan::ProgressiveLoan;
use fineract_progressive_loan::ProgressiveLoanPhase;
use fineract_progressive_loan::ProgressiveLoanRepository;
use fineract_progressive_loan::UpdateProgressiveLoanCommand;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateProgressiveLoanRequest {
    pub client_id: i64,
    pub total_amount: String,
    pub currency: String,
    pub interest_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProgressiveLoanRequest {
    pub client_id: i64,
    pub total_amount: String,
    pub currency: String,
    pub interest_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct AdvancePhaseRequest {
    pub phase: String,
}

#[derive(Debug, Serialize)]
pub struct ProgressiveLoanResponse {
    pub id: i64,
    pub client_id: i64,
    pub total_amount: String,
    pub currency: String,
    pub current_phase: String,
    pub disbursed_amount: String,
    pub remaining_amount: String,
    pub interest_rate: String,
    pub active: bool,
}

/// GET /v1/progressive-loans — list progressive loans for the tenant.
pub async fn list_progressive_loans(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<ProgressiveLoanResponse>>, AppError> {
    let ctx = &auth.0;
    let loans = state
        .progressive_loan_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        loans
            .into_iter()
            .map(|l| ProgressiveLoanResponse {
                id: l.id.value(),
                client_id: l.client_id.value(),
                total_amount: l.total_amount.to_string(),
                currency: l.currency.code(),
                current_phase: format!("{:?}", l.current_phase),
                disbursed_amount: l.disbursed_amount.to_string(),
                remaining_amount: l.remaining_amount.to_string(),
                interest_rate: l.interest_rate.to_string(),
                active: l.active,
            })
            .collect(),
    ))
}

/// GET /v1/progressive-loans/:id — get a single progressive loan.
pub async fn get_progressive_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ProgressiveLoanResponse>, AppError> {
    let ctx = &auth.0;
    let loan = state
        .progressive_loan_repository
        .find_by_id(ctx, LoanId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match loan {
        Some(l) => Ok(Json(ProgressiveLoanResponse {
            id: l.id.value(),
            client_id: l.client_id.value(),
            total_amount: l.total_amount.to_string(),
            currency: l.currency.code(),
            current_phase: format!("{:?}", l.current_phase),
            disbursed_amount: l.disbursed_amount.to_string(),
            remaining_amount: l.remaining_amount.to_string(),
            interest_rate: l.interest_rate.to_string(),
            active: l.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/clients/:client_id/progressive-loans — list progressive loans for a client.
pub async fn list_progressive_loans_by_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(client_id): Path<i64>,
) -> Result<Json<Vec<ProgressiveLoanResponse>>, AppError> {
    let ctx = &auth.0;
    let loans = state
        .progressive_loan_repository
        .find_all_by_client(ctx, ClientId::new(client_id))
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        loans
            .into_iter()
            .map(|l| ProgressiveLoanResponse {
                id: l.id.value(),
                client_id: l.client_id.value(),
                total_amount: l.total_amount.to_string(),
                currency: l.currency.code(),
                current_phase: format!("{:?}", l.current_phase),
                disbursed_amount: l.disbursed_amount.to_string(),
                remaining_amount: l.remaining_amount.to_string(),
                interest_rate: l.interest_rate.to_string(),
                active: l.active,
            })
            .collect(),
    ))
}

/// POST /v1/progressive-loans — create a progressive loan via the command pipeline.
pub async fn create_progressive_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateProgressiveLoanRequest>,
) -> Result<Json<ProgressiveLoanResponse>, AppError> {
    let ctx = &auth.0;
    let total_amount = Decimal::from_str_exact(&payload.total_amount)
        .map_err(|_| AppError::BadRequest("invalid total_amount".to_string()))?;
    let interest_rate = Decimal::from_str_exact(&payload.interest_rate)
        .map_err(|_| AppError::BadRequest("invalid interest_rate".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_progressive_loan::CreateProgressiveLoanCommand {
            tenant_id: ctx.tenant_id,
            client_id: payload.client_id,
            total_amount: total_amount.to_string(),
            currency: payload.currency,
            interest_rate: interest_rate.to_string(),
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let loan_id = result.response.ok_or(AppError::Internal)?;
            let loan = state
                .progressive_loan_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(ProgressiveLoanResponse {
                id: loan.id.value(),
                client_id: loan.client_id.value(),
                total_amount: loan.total_amount.to_string(),
                currency: loan.currency.code(),
                current_phase: format!("{:?}", loan.current_phase),
                disbursed_amount: loan.disbursed_amount.to_string(),
                remaining_amount: loan.remaining_amount.to_string(),
                interest_rate: loan.interest_rate.to_string(),
                active: loan.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/progressive-loans/:id — update a progressive loan.
pub async fn update_progressive_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateProgressiveLoanRequest>,
) -> Result<Json<ProgressiveLoanResponse>, AppError> {
    let ctx = &auth.0;
    let total_amount = Decimal::from_str_exact(&payload.total_amount)
        .map_err(|_| AppError::BadRequest("invalid total_amount".to_string()))?;
    let interest_rate = Decimal::from_str_exact(&payload.interest_rate)
        .map_err(|_| AppError::BadRequest("invalid interest_rate".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_progressive_loan::UpdateProgressiveLoanCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            client_id: payload.client_id,
            total_amount: total_amount.to_string(),
            currency: payload.currency,
            interest_rate: interest_rate.to_string(),
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let loan_id = result.response.ok_or(AppError::Internal)?;
            let loan = state
                .progressive_loan_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(ProgressiveLoanResponse {
                id: loan.id.value(),
                client_id: loan.client_id.value(),
                total_amount: loan.total_amount.to_string(),
                currency: loan.currency.code(),
                current_phase: format!("{:?}", loan.current_phase),
                disbursed_amount: loan.disbursed_amount.to_string(),
                remaining_amount: loan.remaining_amount.to_string(),
                interest_rate: loan.interest_rate.to_string(),
                active: loan.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// POST /v1/progressive-loans/:id/advance-phase — advance the phase of a progressive loan.
pub async fn advance_phase(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<AdvancePhaseRequest>,
) -> Result<Json<ProgressiveLoanResponse>, AppError> {
    let ctx = &auth.0;
    let phase = match payload.phase.as_str() {
        "Application" => ProgressiveLoanPhase::Application,
        "Approved" => ProgressiveLoanPhase::Approved,
        "Disbursed" => ProgressiveLoanPhase::Disbursed,
        "Completed" => ProgressiveLoanPhase::Completed,
        _ => return Err(AppError::BadRequest("invalid phase".to_string())),
    };

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_progressive_loan::AdvancePhaseCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            phase: format!("{:?}", phase),
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let loan_id = result.response.ok_or(AppError::Internal)?;
            let loan = state
                .progressive_loan_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(ProgressiveLoanResponse {
                id: loan.id.value(),
                client_id: loan.client_id.value(),
                total_amount: loan.total_amount.to_string(),
                currency: loan.currency.code(),
                current_phase: format!("{:?}", loan.current_phase),
                disbursed_amount: loan.disbursed_amount.to_string(),
                remaining_amount: loan.remaining_amount.to_string(),
                interest_rate: loan.interest_rate.to_string(),
                active: loan.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/progressive-loans/:id — delete a progressive loan.
pub async fn delete_progressive_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_progressive_loan::DeleteProgressiveLoanCommand {
            id: LoanId::new(id),
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
