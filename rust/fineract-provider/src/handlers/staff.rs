//! Staff HTTP handlers.
//! Staff management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_staff::CreateStaffCommand;
use fineract_staff::DeleteStaffCommand;
use fineract_staff::Staff;
use fineract_staff::StaffRepository;
use fineract_staff::UpdateStaffCommand;

#[derive(Debug, Deserialize)]
pub struct CreateStaffRequest {
    pub office_id: Option<i64>,
    pub firstname: String,
    pub lastname: String,
    pub username: String,
    pub email: Option<String>,
    pub mobile_no: Option<String>,
    pub is_loan_officer: bool,
    pub is_active: bool,
    pub external_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStaffRequest {
    pub office_id: Option<i64>,
    pub firstname: String,
    pub lastname: String,
    pub username: String,
    pub email: Option<String>,
    pub mobile_no: Option<String>,
    pub is_loan_officer: bool,
    pub is_active: bool,
    pub external_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StaffResponse {
    pub id: i64,
    pub office_id: Option<i64>,
    pub firstname: String,
    pub lastname: String,
    pub username: String,
    pub email: Option<String>,
    pub mobile_no: Option<String>,
    pub is_loan_officer: bool,
    pub is_active: bool,
    pub external_id: Option<String>,
    pub full_name: String,
}

/// GET /v1/staff — list staff for the tenant.
pub async fn list_staff(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StaffResponse>>, AppError> {
    let ctx = &auth.0;
    let staff = state
        .staff_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        staff
            .into_iter()
            .map(|s| {
                let external_id = s.external_id.clone();
                let full_name = s.full_name();
                StaffResponse {
                    id: s.id.value(),
                    office_id: s.office_id.map(|id| id.value()),
                    firstname: s.firstname,
                    lastname: s.lastname,
                    username: s.username,
                    email: s.email,
                    mobile_no: s.mobile_no,
                    is_loan_officer: s.is_loan_officer,
                    is_active: s.is_active,
                    external_id,
                    full_name,
                }
            })
            .collect(),
    ))
}

/// GET /v1/staff/:id — get a single staff member.
pub async fn get_staff(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<StaffResponse>, AppError> {
    let ctx = &auth.0;
    let staff = state
        .staff_repository
        .find_by_id(ctx, StaffId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match staff {
        Some(s) => {
            let external_id = s.external_id.clone();
            let full_name = s.full_name();
            Ok(Json(StaffResponse {
                id: s.id.value(),
                office_id: s.office_id.map(|id| id.value()),
                firstname: s.firstname,
                lastname: s.lastname,
                username: s.username,
                email: s.email,
                mobile_no: s.mobile_no,
                is_loan_officer: s.is_loan_officer,
                is_active: s.is_active,
                external_id,
                full_name,
            }))
        }
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/staff/by-username/:username — get a staff member by username.
pub async fn get_staff_by_username(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<StaffResponse>, AppError> {
    let ctx = &auth.0;
    let staff = state
        .staff_repository
        .find_by_username(ctx, &username)
        .await
        .map_err(|_| AppError::Internal)?;
    match staff {
        Some(s) => {
            let external_id = s.external_id.clone();
            let full_name = s.full_name();
            Ok(Json(StaffResponse {
                id: s.id.value(),
                office_id: s.office_id.map(|id| id.value()),
                firstname: s.firstname,
                lastname: s.lastname,
                username: s.username,
                email: s.email,
                mobile_no: s.mobile_no,
                is_loan_officer: s.is_loan_officer,
                is_active: s.is_active,
                external_id,
                full_name,
            }))
        }
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/staff — create a staff member via the command pipeline.
pub async fn create_staff(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateStaffRequest>,
) -> Result<Json<StaffResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_staff::CreateStaffCommand {
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            firstname: payload.firstname,
            lastname: payload.lastname,
            username: payload.username,
            email: payload.email,
            mobile_no: payload.mobile_no,
            is_loan_officer: payload.is_loan_officer,
            is_active: payload.is_active,
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
            let staff_id = result.response.ok_or(AppError::Internal)?;
            let staff = state
                .staff_repository
                .find_by_id(ctx, staff_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            let external_id = staff.external_id.clone();
            let full_name = staff.full_name();
            Ok(Json(StaffResponse {
                id: staff.id.value(),
                office_id: staff.office_id.map(|id| id.value()),
                firstname: staff.firstname,
                lastname: staff.lastname,
                username: staff.username,
                email: staff.email,
                mobile_no: staff.mobile_no,
                is_loan_officer: staff.is_loan_officer,
                is_active: staff.is_active,
                external_id,
                full_name,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/staff/:id — update a staff member.
pub async fn update_staff(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateStaffRequest>,
) -> Result<Json<StaffResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_staff::UpdateStaffCommand {
            id: StaffId::new(id),
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            firstname: payload.firstname,
            lastname: payload.lastname,
            username: payload.username,
            email: payload.email,
            mobile_no: payload.mobile_no,
            is_loan_officer: payload.is_loan_officer,
            is_active: payload.is_active,
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
            let staff_id = result.response.ok_or(AppError::Internal)?;
            let staff = state
                .staff_repository
                .find_by_id(ctx, staff_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            let external_id = staff.external_id.clone();
            let full_name = staff.full_name();
            Ok(Json(StaffResponse {
                id: staff.id.value(),
                office_id: staff.office_id.map(|id| id.value()),
                firstname: staff.firstname,
                lastname: staff.lastname,
                username: staff.username,
                email: staff.email,
                mobile_no: staff.mobile_no,
                is_loan_officer: staff.is_loan_officer,
                is_active: staff.is_active,
                external_id,
                full_name,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/staff/:id — delete a staff member.
pub async fn delete_staff(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_staff::DeleteStaffCommand {
            id: StaffId::new(id),
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
