//! Report repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Report, ReportError, ReportFormat, ReportStatus, ReportType};
use fineract_core::ids::ReportId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type ReportRepositoryResult<T> = Result<T, ReportError>;

#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportRepositoryResult<Option<Report>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ReportRepositoryResult<Vec<Report>>;
    async fn find_by_status(
        &self,
        context: &AuthorizedContext,
        status: ReportStatus,
    ) -> ReportRepositoryResult<Vec<Report>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        report: &Report,
    ) -> ReportRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        report: &Report,
    ) -> ReportRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: ReportId)
        -> ReportRepositoryResult<()>;
}

pub struct SqlxReportRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxReportRepository {
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
    ) -> ReportRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| ReportError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl ReportRepository for SqlxReportRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportRepositoryResult<Option<Report>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, description, report_type, report_format, parameters, status, file_path, generated_by, generated_at, error_message, created_at, updated_at FROM reports WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| ReportError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_report(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ReportRepositoryResult<Vec<Report>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, description, report_type, report_format, parameters, status, file_path, generated_by, generated_at, error_message, created_at, updated_at FROM reports WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| ReportError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_report(&r)).collect()
    }

    async fn find_by_status(
        &self,
        context: &AuthorizedContext,
        status: ReportStatus,
    ) -> ReportRepositoryResult<Vec<Report>> {
        let pool = self.pool_for_context(context).await?;
        let status_str = match status {
            ReportStatus::Pending => "pending",
            ReportStatus::Generating => "generating",
            ReportStatus::Completed => "completed",
            ReportStatus::Failed => "failed",
            ReportStatus::Cancelled => "cancelled",
        };
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, description, report_type, report_format, parameters, status, file_path, generated_by, generated_at, error_message, created_at, updated_at FROM reports WHERE tenant_id = $1 AND status = $2 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .bind(status_str)
        .fetch_all(&pool)
        .await
        .map_err(|e| ReportError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_report(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        report: &Report,
    ) -> ReportRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
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
        sqlx::query(
            "INSERT INTO reports (id, tenant_id, name, description, report_type, report_format, parameters, status, file_path, generated_by, generated_at, error_message, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(report.id.value())
        .bind(report.tenant_id.value())
        .bind(&report.name)
        .bind(&report.description)
        .bind(report_type_str)
        .bind(report_format_str)
        .bind(&report.parameters)
        .bind(status_str)
        .bind(&report.file_path)
        .bind(report.generated_by)
        .bind(report.generated_at)
        .bind(&report.error_message)
        .bind(report.created_at)
        .bind(report.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| ReportError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        report: &Report,
    ) -> ReportRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
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
        let result = sqlx::query(
            "UPDATE reports SET name = $2, description = $3, report_type = $4, report_format = $5, parameters = $6, status = $7, file_path = $8, generated_by = $9, generated_at = $10, error_message = $11, updated_at = $12 WHERE id = $1 AND tenant_id = $13",
        )
        .bind(report.id.value())
        .bind(&report.name)
        .bind(&report.description)
        .bind(report_type_str)
        .bind(report_format_str)
        .bind(&report.parameters)
        .bind(status_str)
        .bind(&report.file_path)
        .bind(report.generated_by)
        .bind(report.generated_at)
        .bind(&report.error_message)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| ReportError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(ReportError::NotFound(report.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM reports WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| ReportError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(ReportError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_report(row: &sqlx::postgres::PgRow) -> ReportRepositoryResult<Report> {
    let report_type_str: String = row.try_get("report_type")?;
    let report_type = match report_type_str.as_str() {
        "portfolio" => ReportType::Portfolio,
        "savings" => ReportType::Savings,
        "loan" => ReportType::Loan,
        "client" => ReportType::Client,
        "accounting" => ReportType::Accounting,
        "delinquency" => ReportType::Delinquency,
        "disbursement" => ReportType::Disbursement,
        "repayment" => ReportType::Repayment,
        "aging" => ReportType::Aging,
        "custom" => ReportType::Custom,
        _ => {
            return Err(ReportError::QueryError {
                detail: format!("invalid report_type in database: {report_type_str}"),
            });
        }
    };

    let report_format_str: String = row.try_get("report_format")?;
    let report_format = match report_format_str.as_str() {
        "pdf" => ReportFormat::Pdf,
        "excel" => ReportFormat::Excel,
        "csv" => ReportFormat::Csv,
        "html" => ReportFormat::Html,
        "json" => ReportFormat::Json,
        _ => {
            return Err(ReportError::QueryError {
                detail: format!("invalid report_format in database: {report_format_str}"),
            });
        }
    };

    let status_str: String = row.try_get("status")?;
    let status = match status_str.as_str() {
        "pending" => ReportStatus::Pending,
        "generating" => ReportStatus::Generating,
        "completed" => ReportStatus::Completed,
        "failed" => ReportStatus::Failed,
        "cancelled" => ReportStatus::Cancelled,
        _ => {
            return Err(ReportError::QueryError {
                detail: format!("invalid status in database: {status_str}"),
            });
        }
    };

    Ok(Report {
        id: ReportId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            ReportError::QueryError {
                detail: "invalid report id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            ReportError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        report_type,
        report_format,
        parameters: row.try_get("parameters")?,
        status,
        file_path: row.try_get("file_path")?,
        generated_by: row.try_get("generated_by")?,
        generated_at: row.try_get("generated_at")?,
        error_message: row.try_get("error_message")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    #![allow(async_fn_in_trait)]
    use super::*;
    use fineract_core::ids::*;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryReportRepository {
        store: Mutex<HashMap<(TenantId, ReportId), Report>>,
    }

    impl Default for InMemoryReportRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryReportRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: ReportId) -> (TenantId, ReportId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl ReportRepository for InMemoryReportRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: ReportId,
        ) -> ReportRepositoryResult<Option<Report>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> ReportRepositoryResult<Vec<Report>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), r)| {
                    if *tid == context.tenant_id {
                        Some(r.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_by_status(
            &self,
            context: &AuthorizedContext,
            status: ReportStatus,
        ) -> ReportRepositoryResult<Vec<Report>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), r)| {
                    if *tid == context.tenant_id && r.status == status {
                        Some(r.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            report: &Report,
        ) -> ReportRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, report.id), report.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            report: &Report,
        ) -> ReportRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, report.id)) {
                store.insert(Self::key(context.tenant_id, report.id), report.clone());
                Ok(())
            } else {
                Err(ReportError::NotFound(report.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: ReportId,
        ) -> ReportRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(ReportError::NotFound(id))
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
    async fn report_repository_is_tenant_isolated() {
        let repo = InMemoryReportRepository::new();
        let report = Report::new(
            ReportId::new(42),
            TenantId::new(1),
            "Test Report".to_string(),
            None,
            ReportType::Portfolio,
            ReportFormat::Pdf,
            None,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &report).await.unwrap();
        let result = repo.find_by_id(&ctx, ReportId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Test Report");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, ReportId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn report_repository_find_by_status() {
        let repo = InMemoryReportRepository::new();
        let report1 = Report::new(
            ReportId::new(1),
            TenantId::new(1),
            "Completed Report".to_string(),
            None,
            ReportType::Portfolio,
            ReportFormat::Pdf,
            None,
        )
        .unwrap();
        let mut report1 = report1;
        report1.status = ReportStatus::Completed;

        let report2 = Report::new(
            ReportId::new(2),
            TenantId::new(1),
            "Pending Report".to_string(),
            None,
            ReportType::Savings,
            ReportFormat::Excel,
            None,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &report1).await.unwrap();
        repo.insert(&ctx, &report2).await.unwrap();

        let completed = repo
            .find_by_status(&ctx, ReportStatus::Completed)
            .await
            .unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].name, "Completed Report");

        let pending = repo
            .find_by_status(&ctx, ReportStatus::Pending)
            .await
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].name, "Pending Report");
    }
}
