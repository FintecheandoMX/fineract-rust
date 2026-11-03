//! Report HTTP handlers.
//! Report management and generation endpoints.

use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::ReportId;
use fineract_core::ids::TenantId;
use fineract_report::ReportRepository;
use fineract_report::{
    CreateReportCommand, DeleteReportCommand, MarkReportCompletedCommand, MarkReportFailedCommand,
    MarkReportGeneratingCommand, Report, ReportFormat, ReportStatus, ReportType,
    UpdateReportCommand,
};

#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub report_format: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReportRequest {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub report_format: String,
    pub parameters: Option<serde_json::Value>,
    pub status: String,
    pub file_path: Option<String>,
    pub generated_by: Option<i64>,
    pub generated_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReportQueryParams {
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub report_format: String,
    pub parameters: Option<serde_json::Value>,
    pub status: String,
    pub file_path: Option<String>,
    pub generated_by: Option<i64>,
    pub generated_at: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

fn report_to_response(report: Report) -> ReportResponse {
    let report_type_str = match report.report_type {
        ReportType::Portfolio => "portfolio",
        ReportType::Savings => "savings",
        ReportType::Loan => "loan",
        ReportType::Client => "client",
        ReportType::Accounting => "accounting",
        ReportType::Delinquency => "delinquency",
        ReportType::Disbursement => "disbursement",
        ReportType::Repayment => "repayment",
        ReportType::Aging => "aging",
        ReportType::Custom => "custom",
    };
    let report_format_str = match report.report_format {
        ReportFormat::Pdf => "pdf",
        ReportFormat::Excel => "excel",
        ReportFormat::Csv => "csv",
        ReportFormat::Html => "html",
        ReportFormat::Json => "json",
    };
    let status_str = match report.status {
        ReportStatus::Pending => "pending",
        ReportStatus::Generating => "generating",
        ReportStatus::Completed => "completed",
        ReportStatus::Failed => "failed",
        ReportStatus::Cancelled => "cancelled",
    };
    ReportResponse {
        id: report.id.value(),
        name: report.name,
        description: report.description,
        report_type: report_type_str.to_string(),
        report_format: report_format_str.to_string(),
        parameters: report.parameters,
        status: status_str.to_string(),
        file_path: report.file_path,
        generated_by: report.generated_by,
        generated_at: report.generated_at.map(|d| d.to_rfc3339()),
        error_message: report.error_message,
        created_at: report.created_at.to_rfc3339(),
        updated_at: report.updated_at.map(|d| d.to_rfc3339()),
    }
}

/// GET /v1/reports — list all reports for the tenant, optionally filtered by status.
pub async fn list_reports(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<ReportQueryParams>,
) -> Result<Json<Vec<ReportResponse>>, AppError> {
    let ctx = &auth.0;

    let reports = if let Some(status_str) = params.status {
        let status = match status_str.to_lowercase().as_str() {
            "pending" => ReportStatus::Pending,
            "generating" => ReportStatus::Generating,
            "completed" => ReportStatus::Completed,
            "failed" => ReportStatus::Failed,
            "cancelled" => ReportStatus::Cancelled,
            _ => return Err(AppError::BadRequest("invalid status value".to_string())),
        };
        state
            .report_repository
            .find_by_status(ctx, status)
            .await
            .map_err(|_| AppError::Internal)?
    } else {
        state
            .report_repository
            .find_all_by_tenant(ctx)
            .await
            .map_err(|_| AppError::Internal)?
    };

    Ok(Json(reports.into_iter().map(report_to_response).collect()))
}

/// GET /v1/reports/:id — get a single report.
pub async fn get_report(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ReportResponse>, AppError> {
    let ctx = &auth.0;
    let report = state
        .report_repository
        .find_by_id(ctx, ReportId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match report {
        Some(r) => Ok(Json(report_to_response(r))),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/reports — create a report via the command pipeline.
pub async fn create_report(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateReportRequest>,
) -> Result<Json<ReportResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        CreateReportCommand {
            tenant_id: ctx.tenant_id,
            name: payload.name,
            description: payload.description,
            report_type: payload.report_type,
            report_format: payload.report_format,
            parameters: payload.parameters,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let report_id = result.response.ok_or(AppError::Internal)?;
            let report = state
                .report_repository
                .find_by_id(ctx, report_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(report_to_response(report)))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/reports/:id — update a report.
pub async fn update_report(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateReportRequest>,
) -> Result<Json<ReportResponse>, AppError> {
    let ctx = &auth.0;

    let generated_at = payload
        .generated_at
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        UpdateReportCommand {
            id: ReportId::new(id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            description: payload.description,
            report_type: payload.report_type,
            report_format: payload.report_format,
            parameters: payload.parameters,
            status: payload.status,
            file_path: payload.file_path,
            generated_by: payload.generated_by,
            generated_at,
            error_message: payload.error_message,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let report_id = result.response.ok_or(AppError::Internal)?;
            let report = state
                .report_repository
                .find_by_id(ctx, report_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(report_to_response(report)))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/reports/:id — delete a report.
pub async fn delete_report(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        DeleteReportCommand {
            id: ReportId::new(id),
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

/// POST /v1/reports/:id/generate — mark report as generating.
pub async fn generate_report(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ReportResponse>, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        MarkReportGeneratingCommand {
            id: ReportId::new(id),
            tenant_id: ctx.tenant_id,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let report_id = result.response.ok_or(AppError::Internal)?;
            let report = state
                .report_repository
                .find_by_id(ctx, report_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(report_to_response(report)))
        }
        _ => Err(AppError::Internal),
    }
}
