//! Job service with authorization enforcement.
//! Every method checks RBAC + data-level authorization before touching data.

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

use crate::domain::{CobError, Job, JobStatus, JobType};
use crate::repository::JobRepository;
use fineract_core::ids::JobId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

/// Result type for job service operations.
pub type JobServiceResult<T> = Result<T, CobError>;

/// Trait for job domain service.
#[async_trait]
pub trait JobService: Send + Sync {
    async fn create_job(&self, context: &AuthorizedContext, job: Job) -> JobServiceResult<Job>;
    async fn get_job(
        &self,
        context: &AuthorizedContext,
        id: JobId,
    ) -> JobServiceResult<Option<Job>>;
    async fn list_jobs(&self, context: &AuthorizedContext) -> JobServiceResult<Vec<Job>>;
    async fn update_job(&self, context: &AuthorizedContext, job: Job) -> JobServiceResult<Job>;
    async fn delete_job(&self, context: &AuthorizedContext, id: JobId) -> JobServiceResult<JobId>;
    async fn enable_job(&self, context: &AuthorizedContext, id: JobId) -> JobServiceResult<()>;
    async fn disable_job(&self, context: &AuthorizedContext, id: JobId) -> JobServiceResult<()>;
}

/// Concrete job service implementation.
pub struct JobServiceImpl<R: JobRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: JobRepository> JobServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: JobRepository> JobService for JobServiceImpl<R> {
    async fn create_job(&self, context: &AuthorizedContext, job: Job) -> JobServiceResult<Job> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;

        if job.tenant_id != context.tenant_id {
            return Err(CobError::TenantIsolationViolation);
        }

        self.repository.insert_job(context, &job).await?;
        Ok(job)
    }

    async fn get_job(
        &self,
        context: &AuthorizedContext,
        id: JobId,
    ) -> JobServiceResult<Option<Job>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;

        let job = self.repository.find_job_by_id(context, id).await?;

        if let Some(ref j) = job {
            if j.tenant_id != context.tenant_id {
                return Err(CobError::TenantIsolationViolation);
            }
        }

        Ok(job)
    }

    async fn list_jobs(&self, context: &AuthorizedContext) -> JobServiceResult<Vec<Job>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_jobs_by_tenant(context).await
    }

    async fn update_job(&self, context: &AuthorizedContext, job: Job) -> JobServiceResult<Job> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;

        if job.tenant_id != context.tenant_id {
            return Err(CobError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_job_by_id(context, job.id)
            .await?
            .ok_or(CobError::NotFound(job.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(CobError::TenantIsolationViolation);
        }

        let mut updated = job;
        updated.updated_at = Utc::now();
        self.repository.insert_job(context, &updated).await?;
        Ok(updated)
    }

    async fn delete_job(&self, context: &AuthorizedContext, id: JobId) -> JobServiceResult<JobId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;

        let job = self
            .repository
            .find_job_by_id(context, id)
            .await?
            .ok_or(CobError::NotFound(id))?;

        if job.tenant_id != context.tenant_id {
            return Err(CobError::TenantIsolationViolation);
        }

        // TODO: implement delete in repository trait
        let _ = job;
        Ok(id)
    }

    async fn enable_job(&self, context: &AuthorizedContext, id: JobId) -> JobServiceResult<()> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;
        self.repository
            .update_job_status(context, id, JobStatus::Active)
            .await
    }

    async fn disable_job(&self, context: &AuthorizedContext, id: JobId) -> JobServiceResult<()> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CobError::Internal {
                detail: e.to_string(),
            })?;
        self.repository
            .update_job_status(context, id, JobStatus::Disabled)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn JobService) {}
    }
}
