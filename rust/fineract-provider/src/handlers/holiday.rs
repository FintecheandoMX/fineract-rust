//! Holiday HTTP handlers.
//! Holiday and calendar management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::HolidayId;
use fineract_core::ids::TenantId;
use fineract_holiday::CreateHolidayCommand;
use fineract_holiday::DeleteHolidayCommand;
use fineract_holiday::Holiday;
use fineract_holiday::HolidayRepository;
use fineract_holiday::UpdateHolidayCommand;

#[derive(Debug, Deserialize)]
pub struct CreateHolidayRequest {
    pub name: String,
    pub holiday_date: String,
    pub is_recurring: bool,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHolidayRequest {
    pub name: String,
    pub holiday_date: String,
    pub is_recurring: bool,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct HolidayResponse {
    pub id: i64,
    pub name: String,
    pub holiday_date: String,
    pub is_recurring: bool,
    pub active: bool,
}

/// GET /v1/holidays — list holidays for the tenant.
pub async fn list_holidays(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<HolidayResponse>>, AppError> {
    let ctx = &auth.0;
    let holidays = state
        .holiday_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        holidays
            .into_iter()
            .map(|h| HolidayResponse {
                id: h.id.value(),
                name: h.name,
                holiday_date: h.holiday_date.to_string(),
                is_recurring: h.is_recurring,
                active: h.active,
            })
            .collect(),
    ))
}

/// GET /v1/holidays/:id — get a single holiday.
pub async fn get_holiday(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<HolidayResponse>, AppError> {
    let ctx = &auth.0;
    let holiday = state
        .holiday_repository
        .find_by_id(ctx, HolidayId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match holiday {
        Some(h) => Ok(Json(HolidayResponse {
            id: h.id.value(),
            name: h.name,
            holiday_date: h.holiday_date.to_string(),
            is_recurring: h.is_recurring,
            active: h.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/holidays/date/:date — get holidays for a specific date.
pub async fn get_holidays_by_date(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(date_str): Path<String>,
) -> Result<Json<Vec<HolidayResponse>>, AppError> {
    let ctx = &auth.0;
    let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
        AppError::BadRequest("invalid date format, expected YYYY-MM-DD".to_string())
    })?;
    let holidays = state
        .holiday_repository
        .find_by_date(ctx, date)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        holidays
            .into_iter()
            .map(|h| HolidayResponse {
                id: h.id.value(),
                name: h.name,
                holiday_date: h.holiday_date.to_string(),
                is_recurring: h.is_recurring,
                active: h.active,
            })
            .collect(),
    ))
}

/// POST /v1/holidays — create a holiday via the command pipeline.
pub async fn create_holiday(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateHolidayRequest>,
) -> Result<Json<HolidayResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_holiday::CreateHolidayCommand {
            tenant_id: ctx.tenant_id,
            name: payload.name,
            holiday_date: payload.holiday_date,
            is_recurring: payload.is_recurring,
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
            let holiday_id = result.response.ok_or(AppError::Internal)?;
            let holiday = state
                .holiday_repository
                .find_by_id(ctx, holiday_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(HolidayResponse {
                id: holiday.id.value(),
                name: holiday.name,
                holiday_date: holiday.holiday_date.to_string(),
                is_recurring: holiday.is_recurring,
                active: holiday.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/holidays/:id — update a holiday.
pub async fn update_holiday(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateHolidayRequest>,
) -> Result<Json<HolidayResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_holiday::UpdateHolidayCommand {
            id: HolidayId::new(id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            holiday_date: payload.holiday_date,
            is_recurring: payload.is_recurring,
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
            let holiday_id = result.response.ok_or(AppError::Internal)?;
            let holiday = state
                .holiday_repository
                .find_by_id(ctx, holiday_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(HolidayResponse {
                id: holiday.id.value(),
                name: holiday.name,
                holiday_date: holiday.holiday_date.to_string(),
                is_recurring: holiday.is_recurring,
                active: holiday.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/holidays/:id — delete a holiday.
pub async fn delete_holiday(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_holiday::DeleteHolidayCommand {
            id: HolidayId::new(id),
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
