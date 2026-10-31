//! COB domain models.
//! Jobs, schedules, executions, and COB runs form the batch processing core.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use fineract_core::ids::JobExecutionId;
use fineract_core::ids::JobId;
use fineract_core::ids::TenantId;

/// Type of scheduled job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobType {
    /// Close of business - daily end-of-day processing.
    CloseOfBusiness,
    /// Interest accrual for savings/loan accounts.
    InterestAccrual,
    /// Charge application (fees, penalties).
    ChargeApplication,
    /// Data archival / cleanup.
    DataArchival,
    /// Custom tenant job.
    Custom(String),
}

/// Status of a job definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Active,
    Disabled,
}

/// Status of a job execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// A scheduled job definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub tenant_id: TenantId,
    pub name: String,
    pub job_type: JobType,
    pub cron_expression: String,
    pub status: JobStatus,
    pub max_retries: u32,
    pub timeout_seconds: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active: bool,
}

impl Job {
    pub fn new(
        id: JobId,
        tenant_id: TenantId,
        name: String,
        job_type: JobType,
        cron_expression: String,
    ) -> Result<Self, CobError> {
        if name.trim().is_empty() {
            return Err(CobError::EmptyName);
        }
        if cron_expression.trim().is_empty() {
            return Err(CobError::EmptyCronExpression);
        }
        let now = Utc::now();
        Ok(Self {
            id,
            tenant_id,
            name,
            job_type,
            cron_expression,
            status: JobStatus::Active,
            max_retries: 3,
            timeout_seconds: 3600,
            created_at: now,
            updated_at: now,
            active: true,
        })
    }
}

/// A single execution of a job within a tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobExecution {
    pub id: JobExecutionId,
    pub job_id: JobId,
    pub tenant_id: TenantId,
    pub status: ExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

impl JobExecution {
    pub fn new(id: JobExecutionId, job_id: JobId, tenant_id: TenantId) -> Self {
        Self {
            id,
            job_id,
            tenant_id,
            status: ExecutionStatus::Pending,
            started_at: None,
            completed_at: None,
            error_message: None,
            retry_count: 0,
        }
    }
}

/// A COB run — a batch of job executions across tenants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CobRun {
    pub id: uuid::Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tenant_ids: Vec<TenantId>,
    pub status: CobRunStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CobRunStatus {
    Running,
    Completed,
    Failed,
}

impl CobRun {
    pub fn new(tenant_ids: Vec<TenantId>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            started_at: Utc::now(),
            completed_at: None,
            tenant_ids,
            status: CobRunStatus::Running,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CobError {
    #[error("job name must not be empty")]
    EmptyName,
    #[error("cron expression must not be empty")]
    EmptyCronExpression,
    #[error("job not found: {0}")]
    NotFound(JobId),
    #[error("job {0} already exists")]
    AlreadyExists(JobId),
    #[error("execution not found: {0}")]
    ExecutionNotFound(JobExecutionId),
    #[error("job {0} is disabled")]
    JobDisabled(JobId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl CobError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, CobError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for CobError {
    fn from(e: sqlx::Error) -> Self {
        CobError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for CobError {
    fn from(e: fineract_security::SecurityError) -> Self {
        CobError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_creation_validates() {
        let result = Job::new(
            JobId::new(1),
            TenantId::new(1),
            "".to_string(),
            JobType::CloseOfBusiness,
            "0 0 * * *".to_string(),
        );
        assert!(matches!(result, Err(CobError::EmptyName)));

        let job = Job::new(
            JobId::new(1),
            TenantId::new(1),
            "Daily COB".to_string(),
            JobType::CloseOfBusiness,
            "0 0 * * *".to_string(),
        )
        .unwrap();
        assert_eq!(job.name, "Daily COB");
        assert_eq!(job.job_type, JobType::CloseOfBusiness);
        assert!(job.active);
    }

    #[test]
    fn execution_status_transitions() {
        let exec = JobExecution::new(JobExecutionId::new(1), JobId::new(1), TenantId::new(1));
        assert_eq!(exec.status, ExecutionStatus::Pending);
        assert_eq!(exec.retry_count, 0);
    }

    #[test]
    fn cob_run_tenant_isolation() {
        let run = CobRun::new(vec![TenantId::new(1), TenantId::new(2)]);
        assert_eq!(run.tenant_ids.len(), 2);
        assert_eq!(run.status, CobRunStatus::Running);
    }
}
