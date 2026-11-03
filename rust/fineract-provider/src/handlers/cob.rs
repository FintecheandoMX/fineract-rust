//! COB HTTP handlers.
//! Job scheduling and execution management endpoints.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_cob::{
    CreateJobCommand, DisableJobCommand, EnableJobCommand, Job, JobRepository, JobStatus, JobType,
    UpdateJobCommand,
};
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::{JobId, TenantId};

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJobRequest {
    pub name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: i64,
    pub name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub status: String,
    pub max_retries: u32,
    pub timeout_seconds: i64,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

fn job_to_response(job: Job) -> JobResponse {
    let job_type_str = match &job.job_type {
        JobType::CloseOfBusiness => "CloseOfBusiness",
        JobType::InterestAccrual => "InterestAccrual",
        JobType::ChargeApplication => "ChargeApplication",
        JobType::DataArchival => "DataArchival",
        JobType::Custom(s) => s.as_str(),
    };
    let status_str = match job.status {
        JobStatus::Active => "Active",
        JobStatus::Disabled => "Disabled",
    };
    JobResponse {
        id: job.id.value(),
        name: job.name,
        job_type: job_type_str.to_string(),
        cron_expression: job.cron_expression,
        status: status_str.to_string(),
        max_retries: job.max_retries,
        timeout_seconds: job.timeout_seconds,
        active: job.active,
        created_at: job.created_at.to_rfc3339(),
        updated_at: job.updated_at.to_rfc3339(),
    }
}

/// GET /v1/jobs — list all jobs for the tenant.
pub async fn list_jobs(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<JobResponse>>, AppError> {
    let ctx = &auth.0;
    let jobs = state
        .job_repository
        .find_jobs_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(jobs.into_iter().map(job_to_response).collect()))
}

/// GET /v1/jobs/:id — get a single job.
pub async fn get_job(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<JobResponse>, AppError> {
    let ctx = &auth.0;
    let job = state
        .job_repository
        .find_job_by_id(ctx, JobId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match job {
        Some(j) => Ok(Json(job_to_response(j))),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/jobs — create a job via the command pipeline.
pub async fn create_job(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateJobRequest>,
) -> Result<Json<JobResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        CreateJobCommand {
            tenant_id: ctx.tenant_id,
            name: payload.name,
            job_type: payload.job_type,
            cron_expression: payload.cron_expression,
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
            let job_id = result.response.ok_or(AppError::Internal)?;
            let job = state
                .job_repository
                .find_job_by_id(ctx, job_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(job_to_response(job)))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/jobs/:id — update a job.
pub async fn update_job(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateJobRequest>,
) -> Result<Json<JobResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        UpdateJobCommand {
            id: JobId::new(id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            job_type: payload.job_type,
            cron_expression: payload.cron_expression,
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
            let job_id = result.response.ok_or(AppError::Internal)?;
            let job = state
                .job_repository
                .find_job_by_id(ctx, job_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(job_to_response(job)))
        }
        _ => Err(AppError::Internal),
    }
}

/// POST /v1/jobs/:id/enable — enable a job.
pub async fn enable_job(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<JobResponse>, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        EnableJobCommand {
            id: JobId::new(id),
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
            let job_id = result.response.ok_or(AppError::Internal)?;
            let job = state
                .job_repository
                .find_job_by_id(ctx, job_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(job_to_response(job)))
        }
        _ => Err(AppError::Internal),
    }
}

/// POST /v1/jobs/:id/disable — disable a job.
pub async fn disable_job(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<JobResponse>, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        DisableJobCommand {
            id: JobId::new(id),
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
            let job_id = result.response.ok_or(AppError::Internal)?;
            let job = state
                .job_repository
                .find_job_by_id(ctx, job_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(job_to_response(job)))
        }
        _ => Err(AppError::Internal),
    }
}
