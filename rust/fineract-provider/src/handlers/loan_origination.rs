//! Loan Origination HTTP handlers.
//! Loan application, review, and approval workflow.

use axum::extract::{Extension, Path, Query, State};
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
use fineract_loan_origination::ApproveLoanApplicationCommand;
use fineract_loan_origination::DeleteLoanApplicationCommand;
use fineract_loan_origination::LoanApplication;
use fineract_loan_origination::LoanApplicationRepository;
use fineract_loan_origination::LoanApplicationStatus;
use fineract_loan_origination::RejectLoanApplicationCommand;
use fineract_loan_origination::SubmitLoanApplicationCommand;
use fineract_loan_origination::UpdateLoanApplicationCommand;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct SubmitLoanApplicationRequest {
    pub office_id: i64,
    pub client_id: i64,
    pub requested_amount: String,
    pub currency: String,
    pub purpose: String,
    pub submitted_by: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLoanApplicationRequest {
    pub office_id: i64,
    pub client_id: i64,
    pub requested_amount: String,
    pub currency: String,
    pub purpose: String,
}

#[derive(Debug, Deserialize)]
pub struct ApproveLoanApplicationRequest {
    pub reviewed_by: i64,
}

#[derive(Debug, Deserialize)]
pub struct RejectLoanApplicationRequest {
    pub reviewed_by: i64,
    pub rejection_reason: String,
}

#[derive(Debug, Serialize)]
pub struct LoanApplicationResponse {
    pub id: i64,
    pub office_id: i64,
    pub client_id: i64,
    pub requested_amount: String,
    pub currency: String,
    pub purpose: String,
    pub status: String,
    pub submitted_by: i64,
    pub reviewed_by: Option<i64>,
    pub submitted_at: String,
    pub reviewed_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub active: bool,
}

/// GET /v1/loan-applications — list loan applications for the tenant.
pub async fn list_loan_applications(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<LoanApplicationResponse>>, AppError> {
    let ctx = &auth.0;
    let apps = state
        .loan_application_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        apps.into_iter()
            .map(|a| LoanApplicationResponse {
                id: a.id.value(),
                office_id: a.office_id.value(),
                client_id: a.client_id.value(),
                requested_amount: a.requested_amount.to_string(),
                currency: a.currency.code(),
                purpose: a.purpose,
                status: format!("{:?}", a.status),
                submitted_by: a.submitted_by.value(),
                reviewed_by: a.reviewed_by.map(|id| id.value()),
                submitted_at: a.submitted_at.to_rfc3339(),
                reviewed_at: a.reviewed_at.map(|dt| dt.to_rfc3339()),
                rejection_reason: a.rejection_reason,
                active: a.active,
            })
            .collect(),
    ))
}

/// GET /v1/loan-applications/:id — get a single loan application.
pub async fn get_loan_application(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<LoanApplicationResponse>, AppError> {
    let ctx = &auth.0;
    let app = state
        .loan_application_repository
        .find_by_id(ctx, LoanId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match app {
        Some(a) => Ok(Json(LoanApplicationResponse {
            id: a.id.value(),
            office_id: a.office_id.value(),
            client_id: a.client_id.value(),
            requested_amount: a.requested_amount.to_string(),
            currency: a.currency.code(),
            purpose: a.purpose,
            status: format!("{:?}", a.status),
            submitted_by: a.submitted_by.value(),
            reviewed_by: a.reviewed_by.map(|id| id.value()),
            submitted_at: a.submitted_at.to_rfc3339(),
            reviewed_at: a.reviewed_at.map(|dt| dt.to_rfc3339()),
            rejection_reason: a.rejection_reason,
            active: a.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/clients/:client_id/loan-applications — list loan applications for a client.
pub async fn list_loan_applications_by_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(client_id): Path<i64>,
) -> Result<Json<Vec<LoanApplicationResponse>>, AppError> {
    let ctx = &auth.0;
    let apps = state
        .loan_application_repository
        .find_all_by_client(ctx, ClientId::new(client_id))
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        apps.into_iter()
            .map(|a| LoanApplicationResponse {
                id: a.id.value(),
                office_id: a.office_id.value(),
                client_id: a.client_id.value(),
                requested_amount: a.requested_amount.to_string(),
                currency: a.currency.code(),
                purpose: a.purpose,
                status: format!("{:?}", a.status),
                submitted_by: a.submitted_by.value(),
                reviewed_by: a.reviewed_by.map(|id| id.value()),
                submitted_at: a.submitted_at.to_rfc3339(),
                reviewed_at: a.reviewed_at.map(|dt| dt.to_rfc3339()),
                rejection_reason: a.rejection_reason,
                active: a.active,
            })
            .collect(),
    ))
}

/// POST /v1/loan-applications — submit a loan application via the command pipeline.
pub async fn submit_loan_application(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<SubmitLoanApplicationRequest>,
) -> Result<Json<LoanApplicationResponse>, AppError> {
    let ctx = &auth.0;
    let requested_amount = Decimal::from_str_exact(&payload.requested_amount)
        .map_err(|_| AppError::BadRequest("invalid requested_amount".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_origination::SubmitLoanApplicationCommand {
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            client_id: payload.client_id,
            requested_amount: requested_amount.to_string(),
            currency: payload.currency,
            purpose: payload.purpose,
            submitted_by: payload.submitted_by,
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
            // Fetch the created application to return full details
            let app = state
                .loan_application_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(LoanApplicationResponse {
                id: app.id.value(),
                office_id: app.office_id.value(),
                client_id: app.client_id.value(),
                requested_amount: app.requested_amount.to_string(),
                currency: app.currency.code(),
                purpose: app.purpose,
                status: format!("{:?}", app.status),
                submitted_by: app.submitted_by.value(),
                reviewed_by: app.reviewed_by.map(|id| id.value()),
                submitted_at: app.submitted_at.to_rfc3339(),
                reviewed_at: app.reviewed_at.map(|dt| dt.to_rfc3339()),
                rejection_reason: app.rejection_reason,
                active: app.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/loan-applications/:id — update a loan application (only if Submitted status).
pub async fn update_loan_application(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateLoanApplicationRequest>,
) -> Result<Json<LoanApplicationResponse>, AppError> {
    let ctx = &auth.0;
    let requested_amount = Decimal::from_str_exact(&payload.requested_amount)
        .map_err(|_| AppError::BadRequest("invalid requested_amount".to_string()))?;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_origination::UpdateLoanApplicationCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            client_id: payload.client_id,
            requested_amount: requested_amount.to_string(),
            currency: payload.currency,
            purpose: payload.purpose,
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
            let app = state
                .loan_application_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(LoanApplicationResponse {
                id: app.id.value(),
                office_id: app.office_id.value(),
                client_id: app.client_id.value(),
                requested_amount: app.requested_amount.to_string(),
                currency: app.currency.code(),
                purpose: app.purpose,
                status: format!("{:?}", app.status),
                submitted_by: app.submitted_by.value(),
                reviewed_by: app.reviewed_by.map(|id| id.value()),
                submitted_at: app.submitted_at.to_rfc3339(),
                reviewed_at: app.reviewed_at.map(|dt| dt.to_rfc3339()),
                rejection_reason: app.rejection_reason,
                active: app.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// POST /v1/loan-applications/:id/approve — approve a loan application.
pub async fn approve_loan_application(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<ApproveLoanApplicationRequest>,
) -> Result<Json<LoanApplicationResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_origination::ApproveLoanApplicationCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            reviewed_by: payload.reviewed_by,
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
            let app = state
                .loan_application_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(LoanApplicationResponse {
                id: app.id.value(),
                office_id: app.office_id.value(),
                client_id: app.client_id.value(),
                requested_amount: app.requested_amount.to_string(),
                currency: app.currency.code(),
                purpose: app.purpose,
                status: format!("{:?}", app.status),
                submitted_by: app.submitted_by.value(),
                reviewed_by: app.reviewed_by.map(|id| id.value()),
                submitted_at: app.submitted_at.to_rfc3339(),
                reviewed_at: app.reviewed_at.map(|dt| dt.to_rfc3339()),
                rejection_reason: app.rejection_reason,
                active: app.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// POST /v1/loan-applications/:id/reject — reject a loan application.
pub async fn reject_loan_application(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<RejectLoanApplicationRequest>,
) -> Result<Json<LoanApplicationResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_origination::RejectLoanApplicationCommand {
            id: LoanId::new(id),
            tenant_id: ctx.tenant_id,
            reviewed_by: payload.reviewed_by,
            rejection_reason: payload.rejection_reason,
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
            let app = state
                .loan_application_repository
                .find_by_id(ctx, loan_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(LoanApplicationResponse {
                id: app.id.value(),
                office_id: app.office_id.value(),
                client_id: app.client_id.value(),
                requested_amount: app.requested_amount.to_string(),
                currency: app.currency.code(),
                purpose: app.purpose,
                status: format!("{:?}", app.status),
                submitted_by: app.submitted_by.value(),
                reviewed_by: app.reviewed_by.map(|id| id.value()),
                submitted_at: app.submitted_at.to_rfc3339(),
                reviewed_at: app.reviewed_at.map(|dt| dt.to_rfc3339()),
                rejection_reason: app.rejection_reason,
                active: app.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/loan-applications/:id — delete a loan application.
pub async fn delete_loan_application(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_loan_origination::DeleteLoanApplicationCommand {
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
