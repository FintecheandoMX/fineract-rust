//! COB engine orchestrator.
//! Coordinates job scheduling, execution, and multi-tenant COB runs.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::domain::{CobError, CobRun, Job, JobStatus};
use crate::repository::{JobExecutionRepository, JobRepository};
use fineract_core::ids::JobId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

/// Result type for engine operations.
pub type EngineResult<T> = Result<T, CobError>;

/// Trait for the COB engine.
#[async_trait]
pub trait CobEngine: Send + Sync {
    async fn authorize(
        &self,
        context: &AuthorizedContext,
        permission: Permission,
    ) -> EngineResult<()>;
    async fn create_job(&self, context: &AuthorizedContext, job: Job) -> EngineResult<JobId>;
    async fn set_job_status(
        &self,
        context: &AuthorizedContext,
        id: JobId,
        status: JobStatus,
    ) -> EngineResult<()>;
    async fn get_due_jobs(
        &self,
        context: &AuthorizedContext,
        now: DateTime<Utc>,
    ) -> EngineResult<Vec<Job>>;
    async fn run_cob_for_tenant(
        &self,
        context: &AuthorizedContext,
        tenant_id: TenantId,
    ) -> EngineResult<CobRun>;
}

/// Concrete COB engine implementation.
pub struct CobEngineImpl<JR: JobRepository, JER: JobExecutionRepository> {
    job_repository: JR,
    execution_repository: JER,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<JR: JobRepository, JER: JobExecutionRepository> CobEngineImpl<JR, JER> {
    pub fn new(
        job_repository: JR,
        execution_repository: JER,
        auth_service: Arc<dyn AuthorizationService>,
    ) -> Self {
        Self {
            job_repository,
            execution_repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<JR: JobRepository, JER: JobExecutionRepository> CobEngine for CobEngineImpl<JR, JER> {
    async fn authorize(
        &self,
        context: &AuthorizedContext,
        permission: Permission,
    ) -> EngineResult<()> {
        self.auth_service
            .check(context, permission)
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;
        Ok(())
    }

    async fn create_job(&self, context: &AuthorizedContext, job: Job) -> EngineResult<JobId> {
        self.authorize(context, Permission::manage_tenants())
            .await?;

        if job.tenant_id != context.tenant_id {
            return Err(CobError::TenantIsolationViolation);
        }

        self.job_repository.insert_job(context, &job).await?;
        Ok(job.id)
    }

    async fn set_job_status(
        &self,
        context: &AuthorizedContext,
        id: JobId,
        status: JobStatus,
    ) -> EngineResult<()> {
        self.authorize(context, Permission::manage_tenants())
            .await?;

        let job = self
            .job_repository
            .find_job_by_id(context, id)
            .await?
            .ok_or(CobError::NotFound(id))?;

        if job.tenant_id != context.tenant_id {
            return Err(CobError::TenantIsolationViolation);
        }

        self.job_repository
            .update_job_status(context, id, status)
            .await
    }

    async fn get_due_jobs(
        &self,
        context: &AuthorizedContext,
        now: DateTime<Utc>,
    ) -> EngineResult<Vec<Job>> {
        self.authorize(context, Permission::view_client()).await?;
        self.job_repository
            .find_due_jobs_by_tenant(context, now)
            .await
    }

    async fn run_cob_for_tenant(
        &self,
        context: &AuthorizedContext,
        tenant_id: TenantId,
    ) -> EngineResult<CobRun> {
        self.authorize(context, Permission::manage_tenants())
            .await?;

        if tenant_id != context.tenant_id {
            return Err(CobError::TenantIsolationViolation);
        }

        let run = CobRun::new(vec![tenant_id]);

        // Find and execute all due jobs for this tenant.
        let due_jobs = self
            .job_repository
            .find_due_jobs_by_tenant(context, Utc::now())
            .await?;

        for job in due_jobs {
            let _ = self.job_repository.find_job_by_id(context, job.id).await?;
            // Execution would be handled by a worker; here we record the run.
            let _ = job;
        }

        Ok(run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn CobEngine) {}
    }
}
