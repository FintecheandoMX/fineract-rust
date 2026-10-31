//! Tokio-based job scheduler.
//! Schedules jobs by cron expression and dispatches them to workers.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::domain::{Job, JobStatus};
use crate::engine::CobEngine;
use fineract_core::ids::JobId;
use fineract_security::AuthorizedContext;

/// Result type for scheduler operations.
pub type SchedulerResult<T> = Result<T, crate::domain::CobError>;

/// Trait for scheduling and dispatching jobs.
#[async_trait]
pub trait JobScheduler: Send + Sync {
    async fn schedule_job(&self, context: &AuthorizedContext, job: Job) -> SchedulerResult<JobId>;
    async fn enable_job(&self, context: &AuthorizedContext, id: JobId) -> SchedulerResult<()>;
    async fn disable_job(&self, context: &AuthorizedContext, id: JobId) -> SchedulerResult<()>;
    async fn get_due_jobs(
        &self,
        context: &AuthorizedContext,
        now: DateTime<Utc>,
    ) -> SchedulerResult<Vec<Job>>;
}

/// Tokio-based scheduler implementation.
pub struct TokioJobScheduler {
    engine: Arc<dyn CobEngine>,
}

impl TokioJobScheduler {
    pub fn new(engine: Arc<dyn CobEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl JobScheduler for TokioJobScheduler {
    async fn schedule_job(&self, context: &AuthorizedContext, job: Job) -> SchedulerResult<JobId> {
        // Authorization: caller must manage jobs.
        self.engine
            .authorize(context, fineract_security::Permission::manage_tenants())
            .await?;

        // Tenant isolation: job must belong to caller's tenant.
        if job.tenant_id != context.tenant_id {
            return Err(crate::domain::CobError::TenantIsolationViolation);
        }

        self.engine.create_job(context, job).await
    }

    async fn enable_job(&self, context: &AuthorizedContext, id: JobId) -> SchedulerResult<()> {
        self.engine
            .authorize(context, fineract_security::Permission::manage_tenants())
            .await?;
        self.engine
            .set_job_status(context, id, JobStatus::Active)
            .await
    }

    async fn disable_job(&self, context: &AuthorizedContext, id: JobId) -> SchedulerResult<()> {
        self.engine
            .authorize(context, fineract_security::Permission::manage_tenants())
            .await?;
        self.engine
            .set_job_status(context, id, JobStatus::Disabled)
            .await
    }

    async fn get_due_jobs(
        &self,
        context: &AuthorizedContext,
        now: DateTime<Utc>,
    ) -> SchedulerResult<Vec<Job>> {
        self.engine
            .authorize(context, fineract_security::Permission::view_client())
            .await?;
        self.engine.get_due_jobs(context, now).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn JobScheduler) {}
    }
}
