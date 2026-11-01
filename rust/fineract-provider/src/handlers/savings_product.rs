//! Savings product HTTP handlers.
//! Savings product management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::SavingsProductId;
use fineract_core::ids::TenantId;
use fineract_savings_product::CreateSavingsProductCommand;
use fineract_savings_product::DeleteSavingsProductCommand;
use fineract_savings_product::SavingsProduct;
use fineract_savings_product::SavingsProductRepository;
use fineract_savings_product::UpdateSavingsProductCommand;

#[derive(Debug, Deserialize)]
pub struct CreateSavingsProductRequest {
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub currency_digits: i32,
    pub interest_rate: String,
    pub interest_compounding_period_type: i32,
    pub interest_posting_period_type: i32,
    pub interest_calculation_type: i32,
    pub interest_calculation_days_in_year_type: i32,
    pub min_required_opening_balance: String,
    pub max_balance: Option<String>,
    pub min_deposit_amount: Option<String>,
    pub max_deposit_amount: Option<String>,
    pub min_withdrawal_amount: Option<String>,
    pub max_withdrawal_amount: Option<String>,
    pub allow_overdraft: bool,
    pub overdraft_limit: Option<String>,
    pub nominal_annual_interest_rate: String,
    pub annual_interest_rate: String,
    pub min_balance_to_earn_interest: Option<String>,
    pub account_fee: Option<String>,
    pub maintenance_fee: Option<String>,
    pub withdrawal_fee: Option<String>,
    pub deposit_fee: Option<String>,
    pub inactive_account_fee: Option<String>,
    pub reject_if_insufficient_funds: bool,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavingsProductRequest {
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub currency_digits: i32,
    pub interest_rate: String,
    pub interest_compounding_period_type: i32,
    pub interest_posting_period_type: i32,
    pub interest_calculation_type: i32,
    pub interest_calculation_days_in_year_type: i32,
    pub min_required_opening_balance: String,
    pub max_balance: Option<String>,
    pub min_deposit_amount: Option<String>,
    pub max_deposit_amount: Option<String>,
    pub min_withdrawal_amount: Option<String>,
    pub max_withdrawal_amount: Option<String>,
    pub allow_overdraft: bool,
    pub overdraft_limit: Option<String>,
    pub nominal_annual_interest_rate: String,
    pub annual_interest_rate: String,
    pub min_balance_to_earn_interest: Option<String>,
    pub account_fee: Option<String>,
    pub maintenance_fee: Option<String>,
    pub withdrawal_fee: Option<String>,
    pub deposit_fee: Option<String>,
    pub inactive_account_fee: Option<String>,
    pub reject_if_insufficient_funds: bool,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct SavingsProductResponse {
    pub id: i64,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub currency_digits: i32,
    pub interest_rate: String,
    pub interest_compounding_period_type: i32,
    pub interest_posting_period_type: i32,
    pub interest_calculation_type: i32,
    pub interest_calculation_days_in_year_type: i32,
    pub min_required_opening_balance: String,
    pub max_balance: Option<String>,
    pub min_deposit_amount: Option<String>,
    pub max_deposit_amount: Option<String>,
    pub min_withdrawal_amount: Option<String>,
    pub max_withdrawal_amount: Option<String>,
    pub allow_overdraft: bool,
    pub overdraft_limit: Option<String>,
    pub nominal_annual_interest_rate: String,
    pub annual_interest_rate: String,
    pub min_balance_to_earn_interest: Option<String>,
    pub account_fee: Option<String>,
    pub maintenance_fee: Option<String>,
    pub withdrawal_fee: Option<String>,
    pub deposit_fee: Option<String>,
    pub inactive_account_fee: Option<String>,
    pub reject_if_insufficient_funds: bool,
    pub is_active: bool,
}

/// GET /v1/savingsproducts — list savings products for the tenant.
pub async fn list_savings_products(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<SavingsProductResponse>>, AppError> {
    let ctx = &auth.0;
    let products = state
        .savings_product_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        products
            .into_iter()
            .map(|p| SavingsProductResponse {
                id: p.id.value(),
                name: p.name,
                short_name: p.short_name,
                description: p.description,
                currency_code: p.currency_code,
                currency_digits: p.currency_digits,
                interest_rate: p.interest_rate,
                interest_compounding_period_type: p.interest_compounding_period_type,
                interest_posting_period_type: p.interest_posting_period_type,
                interest_calculation_type: p.interest_calculation_type,
                interest_calculation_days_in_year_type: p.interest_calculation_days_in_year_type,
                min_required_opening_balance: p.min_required_opening_balance,
                max_balance: p.max_balance,
                min_deposit_amount: p.min_deposit_amount,
                max_deposit_amount: p.max_deposit_amount,
                min_withdrawal_amount: p.min_withdrawal_amount,
                max_withdrawal_amount: p.max_withdrawal_amount,
                allow_overdraft: p.allow_overdraft,
                overdraft_limit: p.overdraft_limit,
                nominal_annual_interest_rate: p.nominal_annual_interest_rate,
                annual_interest_rate: p.annual_interest_rate,
                min_balance_to_earn_interest: p.min_balance_to_earn_interest,
                account_fee: p.account_fee,
                maintenance_fee: p.maintenance_fee,
                withdrawal_fee: p.withdrawal_fee,
                deposit_fee: p.deposit_fee,
                inactive_account_fee: p.inactive_account_fee,
                reject_if_insufficient_funds: p.reject_if_insufficient_funds,
                is_active: p.is_active,
            })
            .collect(),
    ))
}

/// GET /v1/savingsproducts/:id — get a single savings product.
pub async fn get_savings_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SavingsProductResponse>, AppError> {
    let ctx = &auth.0;
    let product = state
        .savings_product_repository
        .find_by_id(ctx, SavingsProductId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match product {
        Some(p) => Ok(Json(SavingsProductResponse {
            id: p.id.value(),
            name: p.name,
            short_name: p.short_name,
            description: p.description,
            currency_code: p.currency_code,
            currency_digits: p.currency_digits,
            interest_rate: p.interest_rate,
            interest_compounding_period_type: p.interest_compounding_period_type,
            interest_posting_period_type: p.interest_posting_period_type,
            interest_calculation_type: p.interest_calculation_type,
            interest_calculation_days_in_year_type: p.interest_calculation_days_in_year_type,
            min_required_opening_balance: p.min_required_opening_balance,
            max_balance: p.max_balance,
            min_deposit_amount: p.min_deposit_amount,
            max_deposit_amount: p.max_deposit_amount,
            min_withdrawal_amount: p.min_withdrawal_amount,
            max_withdrawal_amount: p.max_withdrawal_amount,
            allow_overdraft: p.allow_overdraft,
            overdraft_limit: p.overdraft_limit,
            nominal_annual_interest_rate: p.nominal_annual_interest_rate,
            annual_interest_rate: p.annual_interest_rate,
            min_balance_to_earn_interest: p.min_balance_to_earn_interest,
            account_fee: p.account_fee,
            maintenance_fee: p.maintenance_fee,
            withdrawal_fee: p.withdrawal_fee,
            deposit_fee: p.deposit_fee,
            inactive_account_fee: p.inactive_account_fee,
            reject_if_insufficient_funds: p.reject_if_insufficient_funds,
            is_active: p.is_active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/savingsproducts/name/:name — get a savings product by name.
pub async fn get_savings_product_by_name(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<SavingsProductResponse>, AppError> {
    let ctx = &auth.0;
    let product = state
        .savings_product_repository
        .find_by_name(ctx, &name)
        .await
        .map_err(|_| AppError::Internal)?;
    match product {
        Some(p) => Ok(Json(SavingsProductResponse {
            id: p.id.value(),
            name: p.name,
            short_name: p.short_name,
            description: p.description,
            currency_code: p.currency_code,
            currency_digits: p.currency_digits,
            interest_rate: p.interest_rate,
            interest_compounding_period_type: p.interest_compounding_period_type,
            interest_posting_period_type: p.interest_posting_period_type,
            interest_calculation_type: p.interest_calculation_type,
            interest_calculation_days_in_year_type: p.interest_calculation_days_in_year_type,
            min_required_opening_balance: p.min_required_opening_balance,
            max_balance: p.max_balance,
            min_deposit_amount: p.min_deposit_amount,
            max_deposit_amount: p.max_deposit_amount,
            min_withdrawal_amount: p.min_withdrawal_amount,
            max_withdrawal_amount: p.max_withdrawal_amount,
            allow_overdraft: p.allow_overdraft,
            overdraft_limit: p.overdraft_limit,
            nominal_annual_interest_rate: p.nominal_annual_interest_rate,
            annual_interest_rate: p.annual_interest_rate,
            min_balance_to_earn_interest: p.min_balance_to_earn_interest,
            account_fee: p.account_fee,
            maintenance_fee: p.maintenance_fee,
            withdrawal_fee: p.withdrawal_fee,
            deposit_fee: p.deposit_fee,
            inactive_account_fee: p.inactive_account_fee,
            reject_if_insufficient_funds: p.reject_if_insufficient_funds,
            is_active: p.is_active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/savingsproducts — create a savings product via the command pipeline.
pub async fn create_savings_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateSavingsProductRequest>,
) -> Result<Json<SavingsProductResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings_product::CreateSavingsProductCommand {
            tenant_id: ctx.tenant_id,
            name: payload.name,
            short_name: payload.short_name,
            description: payload.description,
            currency_code: payload.currency_code,
            currency_digits: payload.currency_digits,
            interest_rate: payload.interest_rate,
            interest_compounding_period_type: payload.interest_compounding_period_type,
            interest_posting_period_type: payload.interest_posting_period_type,
            interest_calculation_type: payload.interest_calculation_type,
            interest_calculation_days_in_year_type: payload.interest_calculation_days_in_year_type,
            min_required_opening_balance: payload.min_required_opening_balance,
            max_balance: payload.max_balance,
            min_deposit_amount: payload.min_deposit_amount,
            max_deposit_amount: payload.max_deposit_amount,
            min_withdrawal_amount: payload.min_withdrawal_amount,
            max_withdrawal_amount: payload.max_withdrawal_amount,
            allow_overdraft: payload.allow_overdraft,
            overdraft_limit: payload.overdraft_limit,
            nominal_annual_interest_rate: payload.nominal_annual_interest_rate,
            annual_interest_rate: payload.annual_interest_rate,
            min_balance_to_earn_interest: payload.min_balance_to_earn_interest,
            account_fee: payload.account_fee,
            maintenance_fee: payload.maintenance_fee,
            withdrawal_fee: payload.withdrawal_fee,
            deposit_fee: payload.deposit_fee,
            inactive_account_fee: payload.inactive_account_fee,
            reject_if_insufficient_funds: payload.reject_if_insufficient_funds,
            is_active: payload.is_active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let product_id = result.response.ok_or(AppError::Internal)?;
            let product = state
                .savings_product_repository
                .find_by_id(ctx, product_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(SavingsProductResponse {
                id: product.id.value(),
                name: product.name,
                short_name: product.short_name,
                description: product.description,
                currency_code: product.currency_code,
                currency_digits: product.currency_digits,
                interest_rate: product.interest_rate,
                interest_compounding_period_type: product.interest_compounding_period_type,
                interest_posting_period_type: product.interest_posting_period_type,
                interest_calculation_type: product.interest_calculation_type,
                interest_calculation_days_in_year_type: product
                    .interest_calculation_days_in_year_type,
                min_required_opening_balance: product.min_required_opening_balance,
                max_balance: product.max_balance,
                min_deposit_amount: product.min_deposit_amount,
                max_deposit_amount: product.max_deposit_amount,
                min_withdrawal_amount: product.min_withdrawal_amount,
                max_withdrawal_amount: product.max_withdrawal_amount,
                allow_overdraft: product.allow_overdraft,
                overdraft_limit: product.overdraft_limit,
                nominal_annual_interest_rate: product.nominal_annual_interest_rate,
                annual_interest_rate: product.annual_interest_rate,
                min_balance_to_earn_interest: product.min_balance_to_earn_interest,
                account_fee: product.account_fee,
                maintenance_fee: product.maintenance_fee,
                withdrawal_fee: product.withdrawal_fee,
                deposit_fee: product.deposit_fee,
                inactive_account_fee: product.inactive_account_fee,
                reject_if_insufficient_funds: product.reject_if_insufficient_funds,
                is_active: product.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/savingsproducts/:id — update a savings product.
pub async fn update_savings_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateSavingsProductRequest>,
) -> Result<Json<SavingsProductResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings_product::UpdateSavingsProductCommand {
            id: SavingsProductId::new(id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            short_name: payload.short_name,
            description: payload.description,
            currency_code: payload.currency_code,
            currency_digits: payload.currency_digits,
            interest_rate: payload.interest_rate,
            interest_compounding_period_type: payload.interest_compounding_period_type,
            interest_posting_period_type: payload.interest_posting_period_type,
            interest_calculation_type: payload.interest_calculation_type,
            interest_calculation_days_in_year_type: payload.interest_calculation_days_in_year_type,
            min_required_opening_balance: payload.min_required_opening_balance,
            max_balance: payload.max_balance,
            min_deposit_amount: payload.min_deposit_amount,
            max_deposit_amount: payload.max_deposit_amount,
            min_withdrawal_amount: payload.min_withdrawal_amount,
            max_withdrawal_amount: payload.max_withdrawal_amount,
            allow_overdraft: payload.allow_overdraft,
            overdraft_limit: payload.overdraft_limit,
            nominal_annual_interest_rate: payload.nominal_annual_interest_rate,
            annual_interest_rate: payload.annual_interest_rate,
            min_balance_to_earn_interest: payload.min_balance_to_earn_interest,
            account_fee: payload.account_fee,
            maintenance_fee: payload.maintenance_fee,
            withdrawal_fee: payload.withdrawal_fee,
            deposit_fee: payload.deposit_fee,
            inactive_account_fee: payload.inactive_account_fee,
            reject_if_insufficient_funds: payload.reject_if_insufficient_funds,
            is_active: payload.is_active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let product_id = result.response.ok_or(AppError::Internal)?;
            let product = state
                .savings_product_repository
                .find_by_id(ctx, product_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(SavingsProductResponse {
                id: product.id.value(),
                name: product.name,
                short_name: product.short_name,
                description: product.description,
                currency_code: product.currency_code,
                currency_digits: product.currency_digits,
                interest_rate: product.interest_rate,
                interest_compounding_period_type: product.interest_compounding_period_type,
                interest_posting_period_type: product.interest_posting_period_type,
                interest_calculation_type: product.interest_calculation_type,
                interest_calculation_days_in_year_type: product
                    .interest_calculation_days_in_year_type,
                min_required_opening_balance: product.min_required_opening_balance,
                max_balance: product.max_balance,
                min_deposit_amount: product.min_deposit_amount,
                max_deposit_amount: product.max_deposit_amount,
                min_withdrawal_amount: product.min_withdrawal_amount,
                max_withdrawal_amount: product.max_withdrawal_amount,
                allow_overdraft: product.allow_overdraft,
                overdraft_limit: product.overdraft_limit,
                nominal_annual_interest_rate: product.nominal_annual_interest_rate,
                annual_interest_rate: product.annual_interest_rate,
                min_balance_to_earn_interest: product.min_balance_to_earn_interest,
                account_fee: product.account_fee,
                maintenance_fee: product.maintenance_fee,
                withdrawal_fee: product.withdrawal_fee,
                deposit_fee: product.deposit_fee,
                inactive_account_fee: product.inactive_account_fee,
                reject_if_insufficient_funds: product.reject_if_insufficient_funds,
                is_active: product.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/savingsproducts/:id — delete a savings product.
pub async fn delete_savings_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_savings_product::DeleteSavingsProductCommand {
            id: SavingsProductId::new(id),
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
