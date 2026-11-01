//! Working Capital Loan HTTP handlers.
//! Revolving working capital loans with dynamic drawdown and repayment.

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
use fineract_working_capital_loan::CreateWorkingCapitalLoanCommand;
use fineract_working_capital_loan::DeleteWorkingCapitalLoanCommand;
use fineract_working_capital_loan::UpdateWorkingCapitalLoanCommand;
use fineract_working_capital_loan::UpdateWorkingCapitalLoanStatusCommand;
use fineract_working_capital_loan::WorkingCapitalLoan;
use fineract_working_capital_loan::WorkingCapitalLoanRepository;
use fineract_working_capital_loan::WorkingCapitalLoanStatus;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateWorkingCapitalLoanRequest {
    pub client_id: i64,
    pub credit_limit: String,
    pub currency: String,
    pub interest_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkingCapitalLoanRequest {
    pub client_id: i64,
    pub credit_limit: String,
    pub currency: String,
    pub interest_rate: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkingCapitalLoanStatusRequest {
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct WorkingCapitalLoanResponse {
    pub id: i64,
    pub client_id: i64,
    pub credit_limit: String,
    pub currency: String,
    pub outstanding_balance: String,
    pub available_balance: String,
    pub interest_rate: String,
    pub status: String,
    pub active: bool,
}

/// GET /v1/working-capital-loans — list working capital loans for the tenant.
pub async fn list_working_capital_loans(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<WorkingCapitalLoanResponse>>, AppError> {
    let ctx = &auth.0;
    let loans = state
        .working_capital_loan_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        loans
            .into_iter()
            .map(|l| WorkingCapitalLoanResponse {
                id: l.id.value(),
                client_id: l.client_id.value(),
                credit_limit: l.credit_limit.to_string(),
                currency: l.currency.code(),
                outstanding_balance: l.outstanding_balance.to_string(),
                available_balance: l.available_balance.to_string(),
                interest_rate: l.interest_rate.to_string(),
                status: format!("{:?}", l.status),
                active: l.active,
            })
            .collect(),
    ))
}

/// GET /v1/working-capital-loans/:id — get a single working capital loan.
pub async fn get_working_capital_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<WorkingCapitalLoanResponse>, AppError> {
    let ctx = &auth.0;
    let loan = state
        .working_capital_loan_repository
        .find_by_id(ctx, LoanId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match loan {
        Some(l) => Ok(Json(WorkingCapitalLoanResponse {
            id: l.id.value(),
            client_id: l.client_id.value(),
            credit_limit: l.credit_limit.to_string(),
            currency: l.currency.code(),
            outstanding_balance: l.outstanding_balance.to_string(),
            available_balance: l.available_balance.to_string(),
            interest_rate: l.interest_rate.to_string(),
            status: format!("{:?}", l.status),
            active: l.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/clients/:client_id/working-capital-loans — list working capital loans for a client.
pub async fn list_working_capital_loans_by_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(client_id): Path<i64>,
) -> Result<Json<Vec<WorkingCapitalLoanResponse>>, AppError> {
    let ctx = &auth.0;
    let loans = state
        .working_capital_loan_repository
        .find_all_by_client(ctx, ClientId::new(client_id))
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        loans
            .into_iter()
            .map(|l| WorkingCapitalLoanResponse {
                id: l.id.value(),
                client_id: l.client_id.value(),
                credit_limit: l.credit_limit.to_string(),
                currency: l.currency.code(),
                outstanding_balance: l.outstanding_balance.to_string(),
                available_balance: l.available_balance.to_string(),
                interest_rate: l.interest_rate.to_string(),
                status: format!("{:?}", l.status),
                active: l.active,
            })
            .collect(),
    ))
}

/// POST /v1/working-capital-loans — create a working capital loan via the command pipeline.
pub async fn create_working_capital_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateWorkingCapitalLoanRequest>,
) -> Result<Json<WorkingCapitalLoanResponse>, AppError> {
    let ctx = &auth.0;
    let credit_limit = Decimal::from_str_exact(&payload.credit_limit)
        .map_err(|_| AppError::BadRequest("invalid credit_limit".to_string()))?;
    let interest_rate = Decimal::from_str_exact(&payload.interest_rate)
        .map_err(|_| AppError::BadRequest("invalid interest_rate".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_working_capital_loan::CreateWorkingCapitalLoanCommand {
            tenant_id: ctx.tenant_id,
            client_id: payload.client_id,
            credit_limit: credit_limit.to_string(),
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
                .working_capital_loan_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(WorkingCapitalLoanResponse {
                id: loan.id.value(),
                client_id: loan.client_id.value(),
                credit_limit: loan.credit_limit.to_string(),
                currency: loan.currency.code(),
                outstanding_balance: loan.outstanding_balance.to_string(),
                available_balance: loan.available_balance.to_string(),
                interest_rate: loan.interest_rate.to_string(),
                status: format!("{:?}", loan.status),
                active: loan.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/working-capital-loans/:id — update a working capital loan.
pub async fn update_working_capital_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateWorkingCapitalLoanRequest>,
) -> Result<Json<WorkingCapitalLoanResponse>, AppError> {
    let ctx = &auth.0;
    let credit_limit = Decimal::from_str_exact(&payload.credit_limit)
        .map_err(|_| AppError::BadRequest("invalid credit_limit".to_string()))?;
    let interest_rate = Decimal::from_str_exact(&payload.interest_rate)
        .map_err(|_| AppError::BadRequest("invalid interest_rate".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_working_capital_loan::UpdateWorkingCapitalLoanCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            client_id: payload.client_id,
            credit_limit: credit_limit.to_string(),
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
                .working_capital_loan_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(WorkingCapitalLoanResponse {
                id: loan.id.value(),
                client_id: loan.client_id.value(),
                credit_limit: loan.credit_limit.to_string(),
                currency: loan.currency.code(),
                outstanding_balance: loan.outstanding_balance.to_string(),
                available_balance: loan.available_balance.to_string(),
                interest_rate: loan.interest_rate.to_string(),
                status: format!("{:?}", loan.status),
                active: loan.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/working-capital-loans/:id/status — update the status of a working capital loan.
pub async fn update_working_capital_loan_status(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateWorkingCapitalLoanStatusRequest>,
) -> Result<Json<WorkingCapitalLoanResponse>, AppError> {
    let ctx = &auth.0;
    let status = match payload.status.as_str() {
        "Active" => WorkingCapitalLoanStatus::Active,
        "Suspended" => WorkingCapitalLoanStatus::Suspended,
        "Closed" => WorkingCapitalLoanStatus::Closed,
        _ => return Err(AppError::BadRequest("invalid status".to_string())),
    };

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_working_capital_loan::UpdateWorkingCapitalLoanStatusCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            status: format!("{:?}", status),
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
                .working_capital_loan_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(WorkingCapitalLoanResponse {
                id: loan.id.value(),
                client_id: loan.client_id.value(),
                credit_limit: loan.credit_limit.to_string(),
                currency: loan.currency.code(),
                outstanding_balance: loan.outstanding_balance.to_string(),
                available_balance: loan.available_balance.to_string(),
                interest_rate: loan.interest_rate.to_string(),
                status: format!("{:?}", loan.status),
                active: loan.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/working-capital-loans/:id — delete a working capital loan.
pub async fn delete_working_capital_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_working_capital_loan::DeleteWorkingCapitalLoanCommand {
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
