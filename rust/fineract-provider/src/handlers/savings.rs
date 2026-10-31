//! Savings account HTTP handlers.

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

#[derive(Debug, Deserialize)]
pub struct CreateSavingsAccountRequest {
    pub account_no: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct SavingsAccountResponse {
    pub id: i64,
    pub account_no: String,
    pub status: String,
}

/// GET /v1/savings-accounts — list savings accounts for the tenant.
pub async fn list_savings_accounts(
    Extension(_auth): Extension<AuthContext>,
    _state: State<AppState>,
) -> Result<Json<Vec<SavingsAccountResponse>>, AppError> {
    Err(AppError::NotFound)
}

/// GET /v1/savings-accounts/:id — get a single savings account.
pub async fn get_savings_account(
    Extension(_auth): Extension<AuthContext>,
    _state: State<AppState>,
    Path(_id): Path<i64>,
) -> Result<Json<SavingsAccountResponse>, AppError> {
    Err(AppError::NotFound)
}

/// POST /v1/savings-accounts — create a savings account via the command pipeline.
pub async fn create_savings_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateSavingsAccountRequest>,
) -> Result<Json<SavingsAccountResponse>, AppError> {
    let ctx = &auth.0;
    let account_type = match payload.account_type.as_str() {
        "Demand" => fineract_savings::SavingsAccountType::Demand,
        "Savings" => fineract_savings::SavingsAccountType::Savings,
        "FixedDeposit" => fineract_savings::SavingsAccountType::FixedDeposit,
        _ => return Err(AppError::BadRequest("invalid account type".to_string())),
    };
    let currency = fineract_core::money::Currency::from_code(&payload.currency)
        .map_err(|_| AppError::BadRequest("invalid currency code".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings::CreateSavingsAccountCommand {
            tenant_id: ctx.tenant_id,
            account_no: payload.account_no,
            account_type: format!("{:?}", account_type),
            currency: currency.code().to_string(),
            active: payload.active,
        },
    );

    let _result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    Err(AppError::Internal)
}

/// PUT /v1/savings-accounts/:id — update a savings account.
pub async fn update_savings_account(
    Extension(_auth): Extension<AuthContext>,
    _state: State<AppState>,
    Path(_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    Err(AppError::NotFound)
}

/// DELETE /v1/savings-accounts/:id — delete a savings account.
pub async fn delete_savings_account(
    Extension(_auth): Extension<AuthContext>,
    _state: State<AppState>,
    Path(_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    Err(AppError::NotFound)
}

/// POST /v1/savings-accounts/:id/activate — activate a savings account.
pub async fn activate_savings_account(
    Extension(_auth): Extension<AuthContext>,
    _state: State<AppState>,
    Path(_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    Err(AppError::NotFound)
}
