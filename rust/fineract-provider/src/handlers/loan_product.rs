//! Loan product HTTP handlers.
//! Loan product management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::LoanProductId;
use fineract_core::ids::TenantId;
use fineract_loan_product::CreateLoanProductCommand;
use fineract_loan_product::DeleteLoanProductCommand;
use fineract_loan_product::LoanProduct;
use fineract_loan_product::LoanProductRepository;
use fineract_loan_product::UpdateLoanProductCommand;

#[derive(Debug, Deserialize)]
pub struct CreateLoanProductRequest {
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub principal: String,
    pub number_of_repayments: i32,
    pub repayment_every: i32,
    pub repayment_frequency_type: i32,
    pub amortization_type: i32,
    pub interest_rate_per_period: String,
    pub interest_type: i32,
    pub interest_calculation_period_type: i32,
    pub is_equal_amortization: bool,
    pub min_principal: String,
    pub max_principal: String,
    pub min_number_of_repayments: i32,
    pub max_number_of_repayments: i32,
    pub min_interest_rate_per_period: String,
    pub max_interest_rate_per_period: String,
    pub currency_code: String,
    pub currency_digits: i32,
    pub in_arrears_tolerance: i32,
    pub days_in_year_type: i32,
    pub days_in_month_type: i32,
    pub include_in_borrower_cycle: bool,
    pub use_borrower_cycle: bool,
    pub number_of_borrower_cycle: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLoanProductRequest {
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub principal: String,
    pub number_of_repayments: i32,
    pub repayment_every: i32,
    pub repayment_frequency_type: i32,
    pub amortization_type: i32,
    pub interest_rate_per_period: String,
    pub interest_type: i32,
    pub interest_calculation_period_type: i32,
    pub is_equal_amortization: bool,
    pub min_principal: String,
    pub max_principal: String,
    pub min_number_of_repayments: i32,
    pub max_number_of_repayments: i32,
    pub min_interest_rate_per_period: String,
    pub max_interest_rate_per_period: String,
    pub currency_code: String,
    pub currency_digits: i32,
    pub in_arrears_tolerance: i32,
    pub days_in_year_type: i32,
    pub days_in_month_type: i32,
    pub include_in_borrower_cycle: bool,
    pub use_borrower_cycle: bool,
    pub number_of_borrower_cycle: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct LoanProductResponse {
    pub id: i64,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub principal: String,
    pub number_of_repayments: i32,
    pub repayment_every: i32,
    pub repayment_frequency_type: i32,
    pub amortization_type: i32,
    pub interest_rate_per_period: String,
    pub interest_type: i32,
    pub interest_calculation_period_type: i32,
    pub is_equal_amortization: bool,
    pub min_principal: String,
    pub max_principal: String,
    pub min_number_of_repayments: i32,
    pub max_number_of_repayments: i32,
    pub min_interest_rate_per_period: String,
    pub max_interest_rate_per_period: String,
    pub currency_code: String,
    pub currency_digits: i32,
    pub in_arrears_tolerance: i32,
    pub days_in_year_type: i32,
    pub days_in_month_type: i32,
    pub include_in_borrower_cycle: bool,
    pub use_borrower_cycle: bool,
    pub number_of_borrower_cycle: i32,
    pub is_active: bool,
}

/// GET /v1/loanproducts — list loan products for the tenant.
pub async fn list_loan_products(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LoanProductResponse>>, AppError> {
    let ctx = &auth.0;
    let products = state
        .loan_product_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        products
            .into_iter()
            .map(|p| LoanProductResponse {
                id: p.id.value(),
                name: p.name,
                short_name: p.short_name,
                description: p.description,
                principal: p.principal,
                number_of_repayments: p.number_of_repayments,
                repayment_every: p.repayment_every,
                repayment_frequency_type: p.repayment_frequency_type,
                amortization_type: p.amortization_type,
                interest_rate_per_period: p.interest_rate_per_period,
                interest_type: p.interest_type,
                interest_calculation_period_type: p.interest_calculation_period_type,
                is_equal_amortization: p.is_equal_amortization,
                min_principal: p.min_principal,
                max_principal: p.max_principal,
                min_number_of_repayments: p.min_number_of_repayments,
                max_number_of_repayments: p.max_number_of_repayments,
                min_interest_rate_per_period: p.min_interest_rate_per_period,
                max_interest_rate_per_period: p.max_interest_rate_per_period,
                currency_code: p.currency_code,
                currency_digits: p.currency_digits,
                in_arrears_tolerance: p.in_arrears_tolerance,
                days_in_year_type: p.days_in_year_type,
                days_in_month_type: p.days_in_month_type,
                include_in_borrower_cycle: p.include_in_borrower_cycle,
                use_borrower_cycle: p.use_borrower_cycle,
                number_of_borrower_cycle: p.number_of_borrower_cycle,
                is_active: p.is_active,
            })
            .collect(),
    ))
}

/// GET /v1/loanproducts/:id — get a single loan product.
pub async fn get_loan_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<LoanProductResponse>, AppError> {
    let ctx = &auth.0;
    let product = state
        .loan_product_repository
        .find_by_id(ctx, LoanProductId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match product {
        Some(p) => Ok(Json(LoanProductResponse {
            id: p.id.value(),
            name: p.name,
            short_name: p.short_name,
            description: p.description,
            principal: p.principal,
            number_of_repayments: p.number_of_repayments,
            repayment_every: p.repayment_every,
            repayment_frequency_type: p.repayment_frequency_type,
            amortization_type: p.amortization_type,
            interest_rate_per_period: p.interest_rate_per_period,
            interest_type: p.interest_type,
            interest_calculation_period_type: p.interest_calculation_period_type,
            is_equal_amortization: p.is_equal_amortization,
            min_principal: p.min_principal,
            max_principal: p.max_principal,
            min_number_of_repayments: p.min_number_of_repayments,
            max_number_of_repayments: p.max_number_of_repayments,
            min_interest_rate_per_period: p.min_interest_rate_per_period,
            max_interest_rate_per_period: p.max_interest_rate_per_period,
            currency_code: p.currency_code,
            currency_digits: p.currency_digits,
            in_arrears_tolerance: p.in_arrears_tolerance,
            days_in_year_type: p.days_in_year_type,
            days_in_month_type: p.days_in_month_type,
            include_in_borrower_cycle: p.include_in_borrower_cycle,
            use_borrower_cycle: p.use_borrower_cycle,
            number_of_borrower_cycle: p.number_of_borrower_cycle,
            is_active: p.is_active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/loanproducts/name/:name — get a loan product by name.
pub async fn get_loan_product_by_name(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<LoanProductResponse>, AppError> {
    let ctx = &auth.0;
    let product = state
        .loan_product_repository
        .find_by_name(ctx, &name)
        .await
        .map_err(|_| AppError::Internal)?;
    match product {
        Some(p) => Ok(Json(LoanProductResponse {
            id: p.id.value(),
            name: p.name,
            short_name: p.short_name,
            description: p.description,
            principal: p.principal,
            number_of_repayments: p.number_of_repayments,
            repayment_every: p.repayment_every,
            repayment_frequency_type: p.repayment_frequency_type,
            amortization_type: p.amortization_type,
            interest_rate_per_period: p.interest_rate_per_period,
            interest_type: p.interest_type,
            interest_calculation_period_type: p.interest_calculation_period_type,
            is_equal_amortization: p.is_equal_amortization,
            min_principal: p.min_principal,
            max_principal: p.max_principal,
            min_number_of_repayments: p.min_number_of_repayments,
            max_number_of_repayments: p.max_number_of_repayments,
            min_interest_rate_per_period: p.min_interest_rate_per_period,
            max_interest_rate_per_period: p.max_interest_rate_per_period,
            currency_code: p.currency_code,
            currency_digits: p.currency_digits,
            in_arrears_tolerance: p.in_arrears_tolerance,
            days_in_year_type: p.days_in_year_type,
            days_in_month_type: p.days_in_month_type,
            include_in_borrower_cycle: p.include_in_borrower_cycle,
            use_borrower_cycle: p.use_borrower_cycle,
            number_of_borrower_cycle: p.number_of_borrower_cycle,
            is_active: p.is_active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/loanproducts — create a loan product via the command pipeline.
pub async fn create_loan_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateLoanProductRequest>,
) -> Result<Json<LoanProductResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_product::CreateLoanProductCommand {
            tenant_id: ctx.tenant_id,
            name: payload.name,
            short_name: payload.short_name,
            description: payload.description,
            principal: payload.principal,
            number_of_repayments: payload.number_of_repayments,
            repayment_every: payload.repayment_every,
            repayment_frequency_type: payload.repayment_frequency_type,
            amortization_type: payload.amortization_type,
            interest_rate_per_period: payload.interest_rate_per_period,
            interest_type: payload.interest_type,
            interest_calculation_period_type: payload.interest_calculation_period_type,
            is_equal_amortization: payload.is_equal_amortization,
            min_principal: payload.min_principal,
            max_principal: payload.max_principal,
            min_number_of_repayments: payload.min_number_of_repayments,
            max_number_of_repayments: payload.max_number_of_repayments,
            min_interest_rate_per_period: payload.min_interest_rate_per_period,
            max_interest_rate_per_period: payload.max_interest_rate_per_period,
            currency_code: payload.currency_code,
            currency_digits: payload.currency_digits,
            in_arrears_tolerance: payload.in_arrears_tolerance,
            days_in_year_type: payload.days_in_year_type,
            days_in_month_type: payload.days_in_month_type,
            include_in_borrower_cycle: payload.include_in_borrower_cycle,
            use_borrower_cycle: payload.use_borrower_cycle,
            number_of_borrower_cycle: payload.number_of_borrower_cycle,
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
                .loan_product_repository
                .find_by_id(ctx, product_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(LoanProductResponse {
                id: product.id.value(),
                name: product.name,
                short_name: product.short_name,
                description: product.description,
                principal: product.principal,
                number_of_repayments: product.number_of_repayments,
                repayment_every: product.repayment_every,
                repayment_frequency_type: product.repayment_frequency_type,
                amortization_type: product.amortization_type,
                interest_rate_per_period: product.interest_rate_per_period,
                interest_type: product.interest_type,
                interest_calculation_period_type: product.interest_calculation_period_type,
                is_equal_amortization: product.is_equal_amortization,
                min_principal: product.min_principal,
                max_principal: product.max_principal,
                min_number_of_repayments: product.min_number_of_repayments,
                max_number_of_repayments: product.max_number_of_repayments,
                min_interest_rate_per_period: product.min_interest_rate_per_period,
                max_interest_rate_per_period: product.max_interest_rate_per_period,
                currency_code: product.currency_code,
                currency_digits: product.currency_digits,
                in_arrears_tolerance: product.in_arrears_tolerance,
                days_in_year_type: product.days_in_year_type,
                days_in_month_type: product.days_in_month_type,
                include_in_borrower_cycle: product.include_in_borrower_cycle,
                use_borrower_cycle: product.use_borrower_cycle,
                number_of_borrower_cycle: product.number_of_borrower_cycle,
                is_active: product.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/loanproducts/:id — update a loan product.
pub async fn update_loan_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateLoanProductRequest>,
) -> Result<Json<LoanProductResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_product::UpdateLoanProductCommand {
            id: LoanProductId::new(id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            short_name: payload.short_name,
            description: payload.description,
            principal: payload.principal,
            number_of_repayments: payload.number_of_repayments,
            repayment_every: payload.repayment_every,
            repayment_frequency_type: payload.repayment_frequency_type,
            amortization_type: payload.amortization_type,
            interest_rate_per_period: payload.interest_rate_per_period,
            interest_type: payload.interest_type,
            interest_calculation_period_type: payload.interest_calculation_period_type,
            is_equal_amortization: payload.is_equal_amortization,
            min_principal: payload.min_principal,
            max_principal: payload.max_principal,
            min_number_of_repayments: payload.min_number_of_repayments,
            max_number_of_repayments: payload.max_number_of_repayments,
            min_interest_rate_per_period: payload.min_interest_rate_per_period,
            max_interest_rate_per_period: payload.max_interest_rate_per_period,
            currency_code: payload.currency_code,
            currency_digits: payload.currency_digits,
            in_arrears_tolerance: payload.in_arrears_tolerance,
            days_in_year_type: payload.days_in_year_type,
            days_in_month_type: payload.days_in_month_type,
            include_in_borrower_cycle: payload.include_in_borrower_cycle,
            use_borrower_cycle: payload.use_borrower_cycle,
            number_of_borrower_cycle: payload.number_of_borrower_cycle,
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
                .loan_product_repository
                .find_by_id(ctx, product_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(LoanProductResponse {
                id: product.id.value(),
                name: product.name,
                short_name: product.short_name,
                description: product.description,
                principal: product.principal,
                number_of_repayments: product.number_of_repayments,
                repayment_every: product.repayment_every,
                repayment_frequency_type: product.repayment_frequency_type,
                amortization_type: product.amortization_type,
                interest_rate_per_period: product.interest_rate_per_period,
                interest_type: product.interest_type,
                interest_calculation_period_type: product.interest_calculation_period_type,
                is_equal_amortization: product.is_equal_amortization,
                min_principal: product.min_principal,
                max_principal: product.max_principal,
                min_number_of_repayments: product.min_number_of_repayments,
                max_number_of_repayments: product.max_number_of_repayments,
                min_interest_rate_per_period: product.min_interest_rate_per_period,
                max_interest_rate_per_period: product.max_interest_rate_per_period,
                currency_code: product.currency_code,
                currency_digits: product.currency_digits,
                in_arrears_tolerance: product.in_arrears_tolerance,
                days_in_year_type: product.days_in_year_type,
                days_in_month_type: product.days_in_month_type,
                include_in_borrower_cycle: product.include_in_borrower_cycle,
                use_borrower_cycle: product.use_borrower_cycle,
                number_of_borrower_cycle: product.number_of_borrower_cycle,
                is_active: product.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/loanproducts/:id — delete a loan product.
pub async fn delete_loan_product(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_product::DeleteLoanProductCommand {
            id: LoanProductId::new(id),
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
