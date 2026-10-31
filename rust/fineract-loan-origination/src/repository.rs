//! Loan origination repositories with tenant isolation.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{LoanApplication, LoanApplicationStatus, LoanOriginationError};
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type LoanApplicationRepositoryResult<T> = Result<T, LoanOriginationError>;

#[async_trait]
pub trait LoanApplicationRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanApplicationRepositoryResult<Option<LoanApplication>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> LoanApplicationRepositoryResult<Vec<LoanApplication>>;
    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanApplicationRepositoryResult<Vec<LoanApplication>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        application: &LoanApplication,
    ) -> LoanApplicationRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        application: &LoanApplication,
    ) -> LoanApplicationRepositoryResult<()>;
    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanApplicationStatus,
        reviewed_by: Option<StaffId>,
        rejection_reason: Option<String>,
    ) -> LoanApplicationRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanApplicationRepositoryResult<()>;
}

pub struct SqlxLoanApplicationRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxLoanApplicationRepository {
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
    ) -> LoanApplicationRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| LoanOriginationError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl LoanApplicationRepository for SqlxLoanApplicationRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanApplicationRepositoryResult<Option<LoanApplication>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, office_id, client_id, requested_amount, currency, purpose, status, submitted_by, reviewed_by, submitted_at, reviewed_at, rejection_reason, active FROM loan_applications WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| LoanOriginationError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_application(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> LoanApplicationRepositoryResult<Vec<LoanApplication>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, client_id, requested_amount, currency, purpose, status, submitted_by, reviewed_by, submitted_at, reviewed_at, rejection_reason, active FROM loan_applications WHERE tenant_id = $1 ORDER BY submitted_at DESC",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| LoanOriginationError::QueryError { detail: e.to_string() })?;
        rows.into_iter()
            .map(|r| map_row_to_application(&r))
            .collect()
    }

    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanApplicationRepositoryResult<Vec<LoanApplication>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, client_id, requested_amount, currency, purpose, status, submitted_by, reviewed_by, submitted_at, reviewed_at, rejection_reason, active FROM loan_applications WHERE tenant_id = $1 AND client_id = $2 ORDER BY submitted_at DESC",
        )
        .bind(context.tenant_id.value())
        .bind(client_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| LoanOriginationError::QueryError { detail: e.to_string() })?;
        rows.into_iter()
            .map(|r| map_row_to_application(&r))
            .collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        application: &LoanApplication,
    ) -> LoanApplicationRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO loan_applications (id, tenant_id, office_id, client_id, requested_amount, currency, purpose, status, submitted_by, reviewed_by, submitted_at, reviewed_at, rejection_reason, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(application.id.value())
        .bind(application.tenant_id.value())
        .bind(application.office_id.value())
        .bind(application.client_id.value())
        .bind(application.requested_amount.to_string())
        .bind(application.currency.code())
        .bind(&application.purpose)
        .bind(format!("{:?}", application.status))
        .bind(application.submitted_by.value())
        .bind(application.reviewed_by.map(|id| id.value()))
        .bind(application.submitted_at)
        .bind(application.reviewed_at)
        .bind(&application.rejection_reason)
        .bind(application.active)
        .execute(&pool)
        .await
        .map_err(|e| LoanOriginationError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        application: &LoanApplication,
    ) -> LoanApplicationRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE loan_applications SET office_id = $2, client_id = $3, requested_amount = $4, currency = $5, purpose = $6, status = $7, reviewed_by = $8, submitted_at = $9, reviewed_at = $10, rejection_reason = $11, active = $12 WHERE id = $1 AND tenant_id = $13",
        )
        .bind(application.id.value())
        .bind(application.office_id.value())
        .bind(application.client_id.value())
        .bind(application.requested_amount.to_string())
        .bind(application.currency.code())
        .bind(&application.purpose)
        .bind(format!("{:?}", application.status))
        .bind(application.reviewed_by.map(|id| id.value()))
        .bind(application.submitted_at)
        .bind(application.reviewed_at)
        .bind(&application.rejection_reason)
        .bind(application.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| LoanOriginationError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(LoanOriginationError::NotFound(application.id));
        }
        Ok(())
    }

    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanApplicationStatus,
        reviewed_by: Option<StaffId>,
        rejection_reason: Option<String>,
    ) -> LoanApplicationRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE loan_applications SET status = $2, reviewed_by = $3, reviewed_at = $4, rejection_reason = $5 WHERE id = $1 AND tenant_id = $6",
        )
        .bind(id.value())
        .bind(format!("{:?}", status))
        .bind(reviewed_by.map(|id| id.value()))
        .bind(chrono::Utc::now())
        .bind(&rejection_reason)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| LoanOriginationError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(LoanOriginationError::NotFound(id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanApplicationRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM loan_applications WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| LoanOriginationError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(LoanOriginationError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_application(
    row: &sqlx::postgres::PgRow,
) -> LoanApplicationRepositoryResult<LoanApplication> {
    Ok(LoanApplication {
        id: LoanId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            LoanOriginationError::QueryError {
                detail: "invalid loan id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            LoanOriginationError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        office_id: OfficeId::try_from(row.try_get::<i64, _>("office_id")?).map_err(|_| {
            LoanOriginationError::QueryError {
                detail: "invalid office id".to_string(),
            }
        })?,
        client_id: ClientId::try_from(row.try_get::<i64, _>("client_id")?).map_err(|_| {
            LoanOriginationError::QueryError {
                detail: "invalid client id".to_string(),
            }
        })?,
        requested_amount: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("requested_amount")?,
        )
        .map_err(|_| LoanOriginationError::QueryError {
            detail: "invalid amount".to_string(),
        })?,
        currency: fineract_core::money::Currency::from_code(&row.try_get::<String, _>("currency")?)
            .map_err(|_| LoanOriginationError::QueryError {
                detail: "invalid currency".to_string(),
            })?,
        purpose: row.try_get("purpose")?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "Submitted" => LoanApplicationStatus::Submitted,
            "UnderReview" => LoanApplicationStatus::UnderReview,
            "Approved" => LoanApplicationStatus::Approved,
            "Rejected" => LoanApplicationStatus::Rejected,
            "Withdrawn" => LoanApplicationStatus::Withdrawn,
            _ => {
                return Err(LoanOriginationError::QueryError {
                    detail: "invalid status".to_string(),
                })
            }
        },
        submitted_by: StaffId::try_from(row.try_get::<i64, _>("submitted_by")?).map_err(|_| {
            LoanOriginationError::QueryError {
                detail: "invalid staff id".to_string(),
            }
        })?,
        reviewed_by: row
            .try_get::<Option<i64>, _>("reviewed_by")?
            .map(StaffId::try_from)
            .transpose()
            .map_err(|_| LoanOriginationError::QueryError {
                detail: "invalid staff id".to_string(),
            })?,
        submitted_at: row.try_get("submitted_at")?,
        reviewed_at: row.try_get("reviewed_at")?,
        rejection_reason: row.try_get("rejection_reason")?,
        active: row.try_get("active")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;
    use fineract_core::money::Currency;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryLoanApplicationRepository {
        store: Mutex<HashMap<(TenantId, LoanId), LoanApplication>>,
    }

    impl Default for InMemoryLoanApplicationRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryLoanApplicationRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: LoanId) -> (TenantId, LoanId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl LoanApplicationRepository for InMemoryLoanApplicationRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> LoanApplicationRepositoryResult<Option<LoanApplication>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> LoanApplicationRepositoryResult<Vec<LoanApplication>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), a)| {
                    if *tid == context.tenant_id {
                        Some(a.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_all_by_client(
            &self,
            context: &AuthorizedContext,
            client_id: ClientId,
        ) -> LoanApplicationRepositoryResult<Vec<LoanApplication>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), a)| {
                    if *tid == context.tenant_id && a.client_id == client_id {
                        Some(a.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            application: &LoanApplication,
        ) -> LoanApplicationRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(
                Self::key(context.tenant_id, application.id),
                application.clone(),
            );
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            application: &LoanApplication,
        ) -> LoanApplicationRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, application.id)) {
                store.insert(
                    Self::key(context.tenant_id, application.id),
                    application.clone(),
                );
                Ok(())
            } else {
                Err(LoanOriginationError::NotFound(application.id))
            }
        }

        async fn update_status(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
            status: LoanApplicationStatus,
            reviewed_by: Option<StaffId>,
            rejection_reason: Option<String>,
        ) -> LoanApplicationRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(app) = store.get_mut(&Self::key(context.tenant_id, id)) {
                app.status = status;
                app.reviewed_by = reviewed_by;
                app.rejection_reason = rejection_reason;
                Ok(())
            } else {
                Err(LoanOriginationError::NotFound(id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> LoanApplicationRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(LoanOriginationError::NotFound(id))
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
    async fn loan_application_repository_is_tenant_isolated() {
        let repo = InMemoryLoanApplicationRepository::new();
        let app = LoanApplication::new(
            LoanId::new(42),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::new(50000, 2),
            Currency::USD,
            "Equipment purchase".to_string(),
            StaffId::new(1),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &app).await.unwrap();
        let result = repo.find_by_id(&ctx, LoanId::new(42)).await.unwrap();
        assert_eq!(
            result.unwrap().requested_amount,
            rust_decimal::Decimal::new(50000, 2)
        );

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, LoanId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
