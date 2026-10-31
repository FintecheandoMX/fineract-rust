//! Job and execution repositories with tenant isolation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::domain::{CobError, ExecutionStatus, Job, JobExecution, JobStatus};
use fineract_core::ids::JobExecutionId;
use fineract_core::ids::JobId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type JobRepositoryResult<T> = Result<T, CobError>;

/// Trait for job data access. All operations are tenant-scoped.
#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn find_job_by_id(
        &self,
        context: &AuthorizedContext,
        id: JobId,
    ) -> JobRepositoryResult<Option<Job>>;
    async fn find_jobs_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> JobRepositoryResult<Vec<Job>>;
    async fn find_due_jobs_by_tenant(
        &self,
        context: &AuthorizedContext,
        now: DateTime<Utc>,
    ) -> JobRepositoryResult<Vec<Job>>;
    async fn insert_job(&self, context: &AuthorizedContext, job: &Job) -> JobRepositoryResult<()>;
    async fn update_job_status(
        &self,
        context: &AuthorizedContext,
        id: JobId,
        status: JobStatus,
    ) -> JobRepositoryResult<()>;
}

/// Trait for job execution data access. All operations are tenant-scoped.
#[async_trait]
pub trait JobExecutionRepository: Send + Sync {
    async fn find_execution_by_id(
        &self,
        context: &AuthorizedContext,
        id: JobExecutionId,
    ) -> JobRepositoryResult<Option<JobExecution>>;
    async fn find_executions_by_job(
        &self,
        context: &AuthorizedContext,
        job_id: JobId,
    ) -> JobRepositoryResult<Vec<JobExecution>>;
    async fn create_execution(
        &self,
        context: &AuthorizedContext,
        execution: &JobExecution,
    ) -> JobRepositoryResult<()>;
    async fn update_execution(
        &self,
        context: &AuthorizedContext,
        execution: &JobExecution,
    ) -> JobRepositoryResult<()>;
}

/// SQLx-backed job repository.
pub struct SqlxJobRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxJobRepository {
    pub fn new(
        pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
        connection_string: String,
    ) -> Self {
        Self {
            pool_manager,
            connection_string,
        }
    }

    async fn pool_for_context(
        &self,
        context: &AuthorizedContext,
    ) -> JobRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| CobError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl JobRepository for SqlxJobRepository {
    async fn find_job_by_id(
        &self,
        context: &AuthorizedContext,
        id: JobId,
    ) -> JobRepositoryResult<Option<Job>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, job_type, cron_expression, status, max_retries, timeout_seconds, created_at, updated_at, active FROM jobs WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| CobError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_job(&r)).transpose()
    }

    async fn find_jobs_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> JobRepositoryResult<Vec<Job>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, job_type, cron_expression, status, max_retries, timeout_seconds, created_at, updated_at, active FROM jobs WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| CobError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_job(&r)).collect()
    }

    async fn find_due_jobs_by_tenant(
        &self,
        context: &AuthorizedContext,
        _now: DateTime<Utc>,
    ) -> JobRepositoryResult<Vec<Job>> {
        // TODO: Implement cron-based due-job selection using a cron crate.
        // For now, return all active jobs for the tenant.
        self.find_jobs_by_tenant(context).await
    }

    async fn insert_job(&self, context: &AuthorizedContext, job: &Job) -> JobRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO jobs (id, tenant_id, name, job_type, cron_expression, status, max_retries, timeout_seconds, created_at, updated_at, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(job.id.value())
        .bind(job.tenant_id.value())
        .bind(&job.name)
        .bind(format!("{:?}", job.job_type))
        .bind(&job.cron_expression)
        .bind(format!("{:?}", job.status))
        .bind(job.max_retries as i32)
        .bind(job.timeout_seconds)
        .bind(job.created_at)
        .bind(job.updated_at)
        .bind(job.active)
        .execute(&pool)
        .await
        .map_err(|e| CobError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update_job_status(
        &self,
        context: &AuthorizedContext,
        id: JobId,
        status: JobStatus,
    ) -> JobRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE jobs SET status = $2, updated_at = $3 WHERE id = $1 AND tenant_id = $4",
        )
        .bind(id.value())
        .bind(format!("{:?}", status))
        .bind(Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| CobError::QueryError {
            detail: e.to_string(),
        })?;
        if result.rows_affected() == 0 {
            return Err(CobError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_job(row: &sqlx::postgres::PgRow) -> JobRepositoryResult<Job> {
    Ok(Job {
        id: JobId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| CobError::QueryError {
            detail: "invalid job id".to_string(),
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            CobError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        job_type: match row.try_get::<String, _>("job_type")?.as_str() {
            "CloseOfBusiness" => crate::domain::JobType::CloseOfBusiness,
            "InterestAccrual" => crate::domain::JobType::InterestAccrual,
            "ChargeApplication" => crate::domain::JobType::ChargeApplication,
            "DataArchival" => crate::domain::JobType::DataArchival,
            s => crate::domain::JobType::Custom(s.to_string()),
        },
        cron_expression: row.try_get("cron_expression")?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "Active" => JobStatus::Active,
            "Disabled" => JobStatus::Disabled,
            _ => {
                return Err(CobError::QueryError {
                    detail: "invalid job status".to_string(),
                })
            }
        },
        max_retries: row.try_get::<i32, _>("max_retries")? as u32,
        timeout_seconds: row.try_get("timeout_seconds")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        active: row.try_get("active")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    /// In-memory job repository for testing without a live database.
    pub struct InMemoryJobRepository {
        store: Mutex<HashMap<(TenantId, JobId), Job>>,
    }

    impl Default for InMemoryJobRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryJobRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: JobId) -> (TenantId, JobId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl JobRepository for InMemoryJobRepository {
        async fn find_job_by_id(
            &self,
            context: &AuthorizedContext,
            id: JobId,
        ) -> JobRepositoryResult<Option<Job>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_jobs_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> JobRepositoryResult<Vec<Job>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), j)| {
                    if *tid == context.tenant_id {
                        Some(j.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_due_jobs_by_tenant(
            &self,
            context: &AuthorizedContext,
            _now: DateTime<Utc>,
        ) -> JobRepositoryResult<Vec<Job>> {
            self.find_jobs_by_tenant(context).await
        }

        async fn insert_job(
            &self,
            context: &AuthorizedContext,
            job: &Job,
        ) -> JobRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, job.id), job.clone());
            Ok(())
        }

        async fn update_job_status(
            &self,
            context: &AuthorizedContext,
            id: JobId,
            status: JobStatus,
        ) -> JobRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(job) = store.get_mut(&Self::key(context.tenant_id, id)) {
                job.status = status;
                job.updated_at = chrono::Utc::now();
                Ok(())
            } else {
                Err(CobError::NotFound(id))
            }
        }
    }

    pub struct InMemoryJobExecutionRepository {
        store: Mutex<HashMap<(TenantId, JobExecutionId), JobExecution>>,
    }

    impl Default for InMemoryJobExecutionRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryJobExecutionRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: JobExecutionId) -> (TenantId, JobExecutionId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl JobExecutionRepository for InMemoryJobExecutionRepository {
        async fn find_execution_by_id(
            &self,
            context: &AuthorizedContext,
            id: JobExecutionId,
        ) -> JobRepositoryResult<Option<JobExecution>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_executions_by_job(
            &self,
            context: &AuthorizedContext,
            _job_id: JobId,
        ) -> JobRepositoryResult<Vec<JobExecution>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), e)| {
                    if *tid == context.tenant_id {
                        Some(e.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn create_execution(
            &self,
            context: &AuthorizedContext,
            execution: &JobExecution,
        ) -> JobRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(
                Self::key(context.tenant_id, execution.id),
                execution.clone(),
            );
            Ok(())
        }

        async fn update_execution(
            &self,
            context: &AuthorizedContext,
            execution: &JobExecution,
        ) -> JobRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, execution.id)) {
                store.insert(
                    Self::key(context.tenant_id, execution.id),
                    execution.clone(),
                );
                Ok(())
            } else {
                Err(CobError::ExecutionNotFound(execution.id))
            }
        }
    }

    fn make_context(tenant_id: i64) -> AuthorizedContext {
        AuthorizedContext::new(
            TenantId::new(tenant_id),
            fineract_security::Principal {
                app_user_id: AppUserId::new(1),
                username: "tester".into(),
                roles: HashSet::new(),
                permissions: HashSet::new(),
                office_id: None,
            },
            SessionId::new(),
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn job_repository_is_tenant_isolated() {
        let repo = InMemoryJobRepository::new();
        let job = Job::new(
            JobId::new(42),
            TenantId::new(1),
            "Daily COB".to_string(),
            crate::domain::JobType::CloseOfBusiness,
            "0 0 * * *".to_string(),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert_job(&ctx, &job).await.unwrap();
        let result = repo.find_job_by_id(&ctx, JobId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Daily COB");

        let ctx2 = make_context(2);
        let result = repo.find_job_by_id(&ctx2, JobId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn execution_repository_is_tenant_isolated() {
        let repo = InMemoryJobExecutionRepository::new();
        let exec = JobExecution::new(JobExecutionId::new(42), JobId::new(1), TenantId::new(1));

        let ctx = make_context(1);
        repo.create_execution(&ctx, &exec).await.unwrap();
        let result = repo
            .find_execution_by_id(&ctx, JobExecutionId::new(42))
            .await
            .unwrap();
        assert_eq!(result.unwrap().status, ExecutionStatus::Pending);

        let ctx2 = make_context(2);
        let result = repo
            .find_execution_by_id(&ctx2, JobExecutionId::new(42))
            .await
            .unwrap();
        assert_eq!(result, None);
    }
}
