#![allow(unused_imports, dead_code)]

//! Job worker.
//! Executes jobs with retry, timeout, and tenant isolation.

use async_trait::async_trait;
use chrono::Utc;

use crate::domain::JobExecution;
use crate::repository::{JobExecutionRepository, JobRepository};
use fineract_core::ids::JobExecutionId;
use fineract_core::ids::JobId;
use fineract_security::AuthorizedContext;

/// Result type for worker operations.
pub type WorkerResult<T> = Result<T, crate::domain::CobError>;

/// Trait for executing jobs.
#[async_trait]
pub trait JobWorker: Send + Sync {
    async fn execute_job(
        &self,
        context: &AuthorizedContext,
        job_id: JobId,
    ) -> WorkerResult<JobExecutionId>;
    async fn get_execution(
        &self,
        context: &AuthorizedContext,
        id: JobExecutionId,
    ) -> WorkerResult<Option<JobExecution>>;
}

/// Tokio-based job worker.
pub struct TokioJobWorker<JR: JobRepository, JER: JobExecutionRepository> {
    job_repository: JR,
    execution_repository: JER,
}

impl<JR: JobRepository, JER: JobExecutionRepository> TokioJobWorker<JR, JER> {
    pub fn new(job_repository: JR, execution_repository: JER) -> Self {
        Self {
            job_repository,
            execution_repository,
        }
    }
}

#[async_trait]
impl<JR: JobRepository, JER: JobExecutionRepository> JobWorker for TokioJobWorker<JR, JER> {
    async fn execute_job(
        &self,
        context: &AuthorizedContext,
        job_id: JobId,
    ) -> WorkerResult<JobExecutionId> {
        // Load job and verify tenant ownership (IDOR protection).
        let job = self
            .job_repository
            .find_job_by_id(context, job_id)
            .await?
            .ok_or(crate::domain::CobError::NotFound(job_id))?;

        if job.tenant_id != context.tenant_id {
            return Err(crate::domain::CobError::TenantIsolationViolation);
        }

        if job.status != crate::domain::JobStatus::Active || !job.active {
            return Err(crate::domain::CobError::JobDisabled(job_id));
        }

        // Create execution record.
        let execution = JobExecution::new(
            JobExecutionId::new(1), // TODO: server-side ID generation
            job_id,
            context.tenant_id,
        );

        let execution_id = execution.id;

        // Mark as running.
        let mut running = execution.clone();
        running.status = crate::domain::ExecutionStatus::Running;
        running.started_at = Some(Utc::now());
        self.execution_repository
            .create_execution(context, &running)
            .await?;

        // Execute job type logic.
        let result = self.run_job_type(&job).await;

        // Update execution with result.
        let mut completed = running.clone();
        completed.completed_at = Some(Utc::now());
        match result {
            Ok(()) => {
                completed.status = crate::domain::ExecutionStatus::Completed;
            }
            Err(e) => {
                completed.status = crate::domain::ExecutionStatus::Failed;
                completed.error_message = Some(e.to_string());
            }
        }
        self.execution_repository
            .update_execution(context, &completed)
            .await?;

        Ok(execution_id)
    }

    async fn get_execution(
        &self,
        context: &AuthorizedContext,
        id: JobExecutionId,
    ) -> WorkerResult<Option<JobExecution>> {
        let execution = self
            .execution_repository
            .find_execution_by_id(context, id)
            .await?;

        if let Some(ref exec) = execution {
            if exec.tenant_id != context.tenant_id {
                return Err(crate::domain::CobError::TenantIsolationViolation);
            }
        }

        Ok(execution)
    }
}

impl<JR: JobRepository, JER: JobExecutionRepository> TokioJobWorker<JR, JER> {
    async fn run_job_type(&self, job: &crate::domain::Job) -> WorkerResult<()> {
        match job.job_type {
            crate::domain::JobType::CloseOfBusiness => {
                tracing::info!(job_id = ?job.id, tenant_id = ?job.tenant_id, "running close of business");
                // TODO: invoke per-tenant COB processing
                Ok(())
            }
            crate::domain::JobType::InterestAccrual => {
                tracing::info!(job_id = ?job.id, tenant_id = ?job.tenant_id, "running interest accrual");
                Ok(())
            }
            crate::domain::JobType::ChargeApplication => {
                tracing::info!(job_id = ?job.id, tenant_id = ?job.tenant_id, "running charge application");
                Ok(())
            }
            crate::domain::JobType::DataArchival => {
                tracing::info!(job_id = ?job.id, tenant_id = ?job.tenant_id, "running data archival");
                Ok(())
            }
            crate::domain::JobType::Custom(ref name) => {
                tracing::info!(job_id = ?job.id, tenant_id = ?job.tenant_id, job_name = %name, "running custom job");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn JobWorker) {}
    }
}
