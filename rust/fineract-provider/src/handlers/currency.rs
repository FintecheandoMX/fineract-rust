//! Currency HTTP handlers.
//! Currency management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::CurrencyId;
use fineract_core::ids::TenantId;
use fineract_currency::CreateCurrencyCommand;
use fineract_currency::Currency;
use fineract_currency::CurrencyRepository;
use fineract_currency::DeleteCurrencyCommand;
use fineract_currency::UpdateCurrencyCommand;

#[derive(Debug, Deserialize)]
pub struct CreateCurrencyRequest {
    pub code: String,
    pub name: String,
    pub display_symbol: String,
    pub display_label: String,
    pub decimal_places: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCurrencyRequest {
    pub code: String,
    pub name: String,
    pub display_symbol: String,
    pub display_label: String,
    pub decimal_places: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct CurrencyResponse {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub display_symbol: String,
    pub display_label: String,
    pub decimal_places: i32,
    pub is_active: bool,
}

/// GET /v1/currencies — list currencies for the tenant.
pub async fn list_currencies(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<CurrencyResponse>>, AppError> {
    let ctx = &auth.0;
    let currencies = state
        .currency_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        currencies
            .into_iter()
            .map(|c| CurrencyResponse {
                id: c.id.value(),
                code: c.code,
                name: c.name,
                display_symbol: c.display_symbol,
                display_label: c.display_label,
                decimal_places: c.decimal_places,
                is_active: c.is_active,
            })
            .collect(),
    ))
}

/// GET /v1/currencies/:id — get a single currency.
pub async fn get_currency(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CurrencyResponse>, AppError> {
    let ctx = &auth.0;
    let currency = state
        .currency_repository
        .find_by_id(ctx, CurrencyId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match currency {
        Some(c) => Ok(Json(CurrencyResponse {
            id: c.id.value(),
            code: c.code,
            name: c.name,
            display_symbol: c.display_symbol,
            display_label: c.display_label,
            decimal_places: c.decimal_places,
            is_active: c.is_active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/currencies/code/:code — get a currency by code.
pub async fn get_currency_by_code(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<CurrencyResponse>, AppError> {
    let ctx = &auth.0;
    let currency = state
        .currency_repository
        .find_by_code(ctx, &code)
        .await
        .map_err(|_| AppError::Internal)?;
    match currency {
        Some(c) => Ok(Json(CurrencyResponse {
            id: c.id.value(),
            code: c.code,
            name: c.name,
            display_symbol: c.display_symbol,
            display_label: c.display_label,
            decimal_places: c.decimal_places,
            is_active: c.is_active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/currencies — create a currency via the command pipeline.
pub async fn create_currency(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateCurrencyRequest>,
) -> Result<Json<CurrencyResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_currency::CreateCurrencyCommand {
            tenant_id: ctx.tenant_id,
            code: payload.code,
            name: payload.name,
            display_symbol: payload.display_symbol,
            display_label: payload.display_label,
            decimal_places: payload.decimal_places,
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
            let currency_id = result.response.ok_or(AppError::Internal)?;
            let currency = state
                .currency_repository
                .find_by_id(ctx, currency_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CurrencyResponse {
                id: currency.id.value(),
                code: currency.code,
                name: currency.name,
                display_symbol: currency.display_symbol,
                display_label: currency.display_label,
                decimal_places: currency.decimal_places,
                is_active: currency.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/currencies/:id — update a currency.
pub async fn update_currency(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCurrencyRequest>,
) -> Result<Json<CurrencyResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_currency::UpdateCurrencyCommand {
            id: CurrencyId::new(id),
            tenant_id: ctx.tenant_id,
            code: payload.code,
            name: payload.name,
            display_symbol: payload.display_symbol,
            display_label: payload.display_label,
            decimal_places: payload.decimal_places,
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
            let currency_id = result.response.ok_or(AppError::Internal)?;
            let currency = state
                .currency_repository
                .find_by_id(ctx, currency_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CurrencyResponse {
                id: currency.id.value(),
                code: currency.code,
                name: currency.name,
                display_symbol: currency.display_symbol,
                display_label: currency.display_label,
                decimal_places: currency.decimal_places,
                is_active: currency.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/currencies/:id — delete a currency.
pub async fn delete_currency(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_currency::DeleteCurrencyCommand {
            id: CurrencyId::new(id),
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
