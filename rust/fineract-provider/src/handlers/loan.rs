//! Loan HTTP handlers.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_command::CommandRequest;
use fineract_core::ids::LoanId;
use fineract_core::ids::LoanTransactionId;
use fineract_loan::LoanRepository;

#[derive(Debug, Deserialize)]
pub struct CreateLoanRequest {
    pub client_id: i64,
    pub loan_type: String,
    pub principal: String,
    pub currency: String,
    pub interest_rate: String,
    pub term_count: i32,
    pub term_period: String,
    pub disbursement_date: String,
    pub external_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLoanRequest {
    pub client_id: i64,
    pub loan_type: String,
    pub principal: String,
    pub currency: String,
    pub interest_rate: String,
    pub term_count: i32,
    pub term_period: String,
    pub disbursement_date: String,
    pub external_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordTransactionRequest {
    pub amount: String,
    pub transaction_type: String,
    pub reference: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoanResponse {
    pub id: i64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: i64,
}

/// GET /v1/loans — list loans for the tenant.
pub async fn list_loans(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<LoanResponse>>, AppError> {
    let ctx = &auth.0;
    let loans = state
        .loan_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        loans
            .into_iter()
            .map(|l| LoanResponse {
                id: l.id.value(),
                status: format!("{:?}", l.status),
            })
            .collect(),
    ))
}

/// GET /v1/loans/:id — get a single loan.
pub async fn get_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<LoanResponse>, AppError> {
    let ctx = &auth.0;
    let loan = state
        .loan_repository
        .find_by_id(ctx, LoanId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match loan {
        Some(l) => Ok(Json(LoanResponse {
            id: l.id.value(),
            status: format!("{:?}", l.status),
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/loans — create a loan via the command pipeline.
pub async fn create_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateLoanRequest>,
) -> Result<Json<LoanResponse>, AppError> {
    let ctx = &auth.0;
    let loan_type = match payload.loan_type.as_str() {
        "Individual" => fineract_loan::LoanType::Individual,
        "Group" => fineract_loan::LoanType::Group,
        _ => return Err(AppError::BadRequest("invalid loan type".to_string())),
    };
    let currency = fineract_core::money::Currency::from_code(&payload.currency)
        .map_err(|_| AppError::BadRequest("invalid currency code".to_string()))?;
    let principal = rust_decimal::Decimal::from_str_exact(&payload.principal)
        .map_err(|_| AppError::BadRequest("invalid principal".to_string()))?;
    let interest_rate = rust_decimal::Decimal::from_str_exact(&payload.interest_rate)
        .map_err(|_| AppError::BadRequest("invalid interest rate".to_string()))?;
    let term_period = match payload.term_period.as_str() {
        "Days" => fineract_loan::LoanTermPeriod::Days,
        "Weeks" => fineract_loan::LoanTermPeriod::Weeks,
        "Months" => fineract_loan::LoanTermPeriod::Months,
        "Years" => fineract_loan::LoanTermPeriod::Years,
        _ => return Err(AppError::BadRequest("invalid term period".to_string())),
    };
    let disbursement_date =
        chrono::NaiveDate::parse_from_str(&payload.disbursement_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("invalid disbursement date".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan::CreateLoanCommand {
            tenant_id: ctx.tenant_id,
            office_id: 1,
            client_id: payload.client_id,
            loan_type: format!("{:?}", loan_type),
            principal: principal.to_string(),
            currency: currency.code().to_string(),
            interest_rate: interest_rate.to_string(),
            term_count: payload.term_count,
            term_period: format!("{:?}", term_period),
            disbursement_date: disbursement_date.format("%Y-%m-%d").to_string(),
            external_id: payload.external_id,
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
            Ok(Json(LoanResponse {
                id: loan_id.value(),
                status: "PendingApproval".to_string(),
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/loans/:id — update a loan.
pub async fn update_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateLoanRequest>,
) -> Result<Json<LoanResponse>, AppError> {
    let ctx = &auth.0;
    let loan_type = match payload.loan_type.as_str() {
        "Individual" => fineract_loan::LoanType::Individual,
        "Group" => fineract_loan::LoanType::Group,
        _ => return Err(AppError::BadRequest("invalid loan type".to_string())),
    };
    let currency = fineract_core::money::Currency::from_code(&payload.currency)
        .map_err(|_| AppError::BadRequest("invalid currency code".to_string()))?;
    let principal = rust_decimal::Decimal::from_str_exact(&payload.principal)
        .map_err(|_| AppError::BadRequest("invalid principal".to_string()))?;
    let interest_rate = rust_decimal::Decimal::from_str_exact(&payload.interest_rate)
        .map_err(|_| AppError::BadRequest("invalid interest rate".to_string()))?;
    let term_period = match payload.term_period.as_str() {
        "Days" => fineract_loan::LoanTermPeriod::Days,
        "Weeks" => fineract_loan::LoanTermPeriod::Weeks,
        "Months" => fineract_loan::LoanTermPeriod::Months,
        "Years" => fineract_loan::LoanTermPeriod::Years,
        _ => return Err(AppError::BadRequest("invalid term period".to_string())),
    };
    let disbursement_date =
        chrono::NaiveDate::parse_from_str(&payload.disbursement_date, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("invalid disbursement date".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan::UpdateLoanCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            office_id: 1,
            client_id: payload.client_id,
            loan_type: format!("{:?}", loan_type),
            principal: principal.to_string(),
            currency: currency.code().to_string(),
            interest_rate: interest_rate.to_string(),
            term_count: payload.term_count,
            term_period: format!("{:?}", term_period),
            disbursement_date: disbursement_date.format("%Y-%m-%d").to_string(),
            external_id: payload.external_id,
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
            Ok(Json(LoanResponse {
                id: loan_id.value(),
                status: "PendingApproval".to_string(),
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/loans/:id — delete a loan.
pub async fn delete_loan(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan::DeleteLoanCommand {
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

/// POST /v1/loans/:id/transactions — record a loan transaction.
pub async fn record_loan_transaction(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<RecordTransactionRequest>,
) -> Result<Json<TransactionResponse>, AppError> {
    let ctx = &auth.0;
    let amount = rust_decimal::Decimal::from_str_exact(&payload.amount)
        .map_err(|_| AppError::BadRequest("invalid amount".to_string()))?;
    let transaction_type = match payload.transaction_type.as_str() {
        "Disbursement" => fineract_loan::LoanTransactionType::Disbursement,
        "Repayment" => fineract_loan::LoanTransactionType::Repayment,
        "Fee" => fineract_loan::LoanTransactionType::Fee,
        "Interest" => fineract_loan::LoanTransactionType::Interest,
        "Reversal" => fineract_loan::LoanTransactionType::Reversal,
        "WriteOff" => fineract_loan::LoanTransactionType::WriteOff,
        _ => return Err(AppError::BadRequest("invalid transaction type".to_string())),
    };

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan::RecordTransactionCommand {
            loan_id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            amount: amount.to_string(),
            transaction_type: format!("{:?}", transaction_type),
            reference: payload.reference,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let tx_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(TransactionResponse { id: tx_id.value() }))
        }
        _ => Err(AppError::Internal),
    }
}
