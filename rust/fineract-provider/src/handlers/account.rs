//! Accounting HTTP handlers.
//! GL accounts, journal entries, and accounting periods.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use chrono::Utc;
use fineract_accounting::Account;
use fineract_accounting::AccountRepository;
use fineract_accounting::AccountStatus;
use fineract_accounting::AccountType;
use fineract_accounting::AccountingPeriodRepository;
use fineract_accounting::CreateAccountCommand;
use fineract_accounting::DeleteAccountCommand;
use fineract_accounting::JournalEntryRepository;
use fineract_accounting::UpdateAccountCommand;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_command::CommandRequest;
use fineract_core::ids::AccountId;
use fineract_core::ids::AccountingPeriodId;
use fineract_core::ids::JournalEntryId;
use fineract_core::money::Currency;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: i64,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub currency: String,
    pub status: String,
    pub balance: String,
    pub active: bool,
}

/// GET /v1/accounts — list accounts for the tenant.
pub async fn list_accounts(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let ctx = &auth.0;
    let accounts = state
        .account_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        accounts
            .into_iter()
            .map(|a| AccountResponse {
                id: a.id.value(),
                account_code: a.account_code,
                account_name: a.account_name,
                account_type: format!("{:?}", a.account_type),
                currency: a.currency.code().to_string(),
                status: format!("{:?}", a.status),
                balance: a.balance.to_string(),
                active: a.active,
            })
            .collect(),
    ))
}

/// GET /v1/accounts/:id — get a single account.
pub async fn get_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<AccountResponse>, AppError> {
    let ctx = &auth.0;
    let account = state
        .account_repository
        .find_by_id(ctx, AccountId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match account {
        Some(a) => Ok(Json(AccountResponse {
            id: a.id.value(),
            account_code: a.account_code,
            account_name: a.account_name,
            account_type: format!("{:?}", a.account_type),
            currency: a.currency.code().to_string(),
            status: format!("{:?}", a.status),
            balance: a.balance.to_string(),
            active: a.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/accounts — create an account via the command pipeline.
pub async fn create_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    let ctx = &auth.0;
    let account_type = match payload.account_type.as_str() {
        "Asset" => AccountType::Asset,
        "Liability" => AccountType::Liability,
        "Equity" => AccountType::Equity,
        "Revenue" => AccountType::Revenue,
        "Expense" => AccountType::Expense,
        _ => return Err(AppError::BadRequest("invalid account type".to_string())),
    };
    let currency = fineract_core::money::Currency::from_code(&payload.currency)
        .map_err(|_| AppError::BadRequest("invalid currency code".to_string()))?;

    let account_code = payload.account_code.clone();
    let account_name = payload.account_name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_accounting::CreateAccountCommand {
            tenant_id: ctx.tenant_id,
            account_code: account_code.clone(),
            account_name: payload.account_name,
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
            Ok(Json(AccountResponse {
                id: account_id.value(),
                account_code,
                account_name,
                account_type: format!("{:?}", account_type),
                currency: payload.currency,
                status: "Active".to_string(),
                balance: "0".to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/accounts/:id — update an account.
pub async fn update_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    let ctx = &auth.0;
    let account_type = match payload.account_type.as_str() {
        "Asset" => fineract_accounting::AccountType::Asset,
        "Liability" => fineract_accounting::AccountType::Liability,
        "Equity" => fineract_accounting::AccountType::Equity,
        "Revenue" => fineract_accounting::AccountType::Revenue,
        "Expense" => fineract_accounting::AccountType::Expense,
        _ => return Err(AppError::BadRequest("invalid account type".to_string())),
    };
    let currency = fineract_core::money::Currency::from_code(&payload.currency)
        .map_err(|_| AppError::BadRequest("invalid currency code".to_string()))?;

    let account_code = payload.account_code.clone();
    let account_name = payload.account_name.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_accounting::UpdateAccountCommand {
            id: AccountId::new(id),
            tenant_id: ctx.tenant_id,
            account_code: account_code.clone(),
            account_name: account_name.clone(),
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
            Ok(Json(AccountResponse {
                id: account_id.value(),
                account_code,
                account_name,
                account_type: format!("{:?}", account_type),
                currency: payload.currency,
                status: "Active".to_string(),
                balance: "0".to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/accounts/:id — delete an account.
pub async fn delete_account(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_accounting::DeleteAccountCommand {
            id: AccountId::new(id),
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
