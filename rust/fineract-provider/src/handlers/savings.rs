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
use fineract_core::ids::SavingsAccountId;
use fineract_savings::SavingsAccountRepository;

#[derive(Debug, Deserialize)]
pub struct CreateSavingsAccountRequest {
    pub account_no: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavingsAccountRequest {
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
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<SavingsAccountResponse>>, AppError> {
    let ctx = &auth.0;
    let accounts = state
        .savings_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        accounts
            .into_iter()
            .map(|a| SavingsAccountResponse {
                id: a.id.value(),
                account_no: a.account_no,
                status: format!("{:?}", a.status),
            })
            .collect(),
    ))
}

/// GET /v1/savings-accounts/:id — get a single savings account.
pub async fn get_savings_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SavingsAccountResponse>, AppError> {
    let ctx = &auth.0;
    let account = state
        .savings_repository
        .find_by_id(ctx, SavingsAccountId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match account {
        Some(a) => Ok(Json(SavingsAccountResponse {
            id: a.id.value(),
            account_no: a.account_no,
            status: format!("{:?}", a.status),
        })),
        None => Err(AppError::NotFound),
    }
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

    let account_no = payload.account_no.clone();
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings::CreateSavingsAccountCommand {
            tenant_id: ctx.tenant_id,
            account_no: account_no.clone(),
            account_type: format!("{:?}", account_type),
            currency: currency.code().to_string(),
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
            let account_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(SavingsAccountResponse {
                id: account_id.value(),
                account_no,
                status: "PendingApproval".to_string(),
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/savings-accounts/:id — update a savings account.
pub async fn update_savings_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateSavingsAccountRequest>,
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

    let account_no = payload.account_no.clone();
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings::UpdateSavingsAccountCommand {
            id: SavingsAccountId::new(id),
            tenant_id: ctx.tenant_id,
            account_no: account_no.clone(),
            account_type: format!("{:?}", account_type),
            currency: currency.code().to_string(),
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
            let account_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(SavingsAccountResponse {
                id: account_id.value(),
                account_no,
                status: "PendingApproval".to_string(),
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/savings-accounts/:id — delete a savings account.
pub async fn delete_savings_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings::DeleteSavingsAccountCommand {
            id: SavingsAccountId::new(id),
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

/// POST /v1/savings-accounts/:id/activate — activate a savings account.
pub async fn activate_savings_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings::ActivateSavingsAccountCommand {
            id: SavingsAccountId::new(id),
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
