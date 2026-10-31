//! Branch repository trait and implementations.
//!
//! All queries are parameterized via sqlx and filtered by tenant_id by
//! default, enforcing multi-tenancy isolation at the repository layer.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Office, OfficeError};
use fineract_core::ids::OfficeId;
use fineract_core::ids::TenantId;

use fineract_security::AuthorizedContext;

/// Result type for branch repository operations.
pub type BranchRepositoryResult<T> = Result<T, OfficeError>;

/// Trait for branch data access.
///
/// All operations are tenant-scoped: the context's tenant_id is used to
/// filter rows, preventing cross-tenant data access (IDOR protection).
#[async_trait]
pub trait BranchRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: OfficeId,
    ) -> BranchRepositoryResult<Option<Office>>;

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> BranchRepositoryResult<Vec<Office>>;

    async fn insert(
        &self,
        context: &AuthorizedContext,
        office: &Office,
    ) -> BranchRepositoryResult<()>;

    async fn update(
        &self,
        context: &AuthorizedContext,
        office: &Office,
    ) -> BranchRepositoryResult<()>;

    async fn delete(&self, context: &AuthorizedContext, id: OfficeId)
        -> BranchRepositoryResult<()>;
}

/// SQLx-backed branch repository.
pub struct SqlxBranchRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxBranchRepository {
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
    ) -> BranchRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| OfficeError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl BranchRepository for SqlxBranchRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: OfficeId,
    ) -> BranchRepositoryResult<Option<Office>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, external_id, opening_date, timezone, country, currency, parent_office_id, active FROM offices WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| OfficeError::QueryError {
            detail: e.to_string(),
        })?;

        match row {
            Some(r) => Ok(Some(map_row_to_office(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> BranchRepositoryResult<Vec<Office>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, external_id, opening_date, timezone, country, currency, parent_office_id, active FROM offices WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| OfficeError::QueryError {
            detail: e.to_string(),
        })?;

        rows.into_iter().map(|r| map_row_to_office(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        office: &Office,
    ) -> BranchRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO offices (id, tenant_id, name, external_id, opening_date, timezone, country, currency, parent_office_id, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(office.id.value())
        .bind(office.tenant_id.value())
        .bind(&office.name)
        .bind(&office.external_id)
        .bind(office.opening_date)
        .bind(&office.timezone)
        .bind(&office.country)
        .bind(office.currency.code())
        .bind(office.parent_office_id.map(|id| id.value()))
        .bind(office.active)
        .execute(&pool)
        .await
        .map_err(|e| OfficeError::QueryError {
            detail: e.to_string(),
        })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        office: &Office,
    ) -> BranchRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE offices SET name = $2, external_id = $3, opening_date = $4, timezone = $5, country = $6, currency = $7, parent_office_id = $8, active = $9 WHERE id = $1 AND tenant_id = $10",
        )
        .bind(office.id.value())
        .bind(&office.name)
        .bind(&office.external_id)
        .bind(office.opening_date)
        .bind(&office.timezone)
        .bind(&office.country)
        .bind(office.currency.code())
        .bind(office.parent_office_id.map(|id| id.value()))
        .bind(office.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| OfficeError::QueryError {
            detail: e.to_string(),
        })?;

        if result.rows_affected() == 0 {
            return Err(OfficeError::NotFound(office.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: OfficeId,
    ) -> BranchRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM offices WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| OfficeError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(OfficeError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_office(row: &sqlx::postgres::PgRow) -> BranchRepositoryResult<Office> {
    Ok(Office {
        id: OfficeId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            OfficeError::QueryError {
                detail: "invalid office id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            OfficeError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        external_id: row.try_get("external_id")?,
        opening_date: row.try_get("opening_date")?,
        timezone: row.try_get("timezone")?,
        country: row.try_get("country")?,
        currency: fineract_core::money::Currency::from_code(&row.try_get::<String, _>("currency")?)
            .map_err(|_| OfficeError::QueryError {
                detail: "invalid currency code in database".to_string(),
            })?,
        parent_office_id: row
            .try_get::<Option<i64>, _>("parent_office_id")?
            .map(OfficeId::try_from)
            .transpose()
            .map_err(|_| OfficeError::QueryError {
                detail: "invalid parent office id in database".to_string(),
            })?,
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

    /// In-memory branch repository for testing without a live database.
    pub struct InMemoryBranchRepository {
        store: Mutex<HashMap<(TenantId, OfficeId), Office>>,
    }

    impl Default for InMemoryBranchRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryBranchRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: OfficeId) -> (TenantId, OfficeId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl BranchRepository for InMemoryBranchRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: OfficeId,
        ) -> BranchRepositoryResult<Option<Office>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> BranchRepositoryResult<Vec<Office>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), office)| {
                    if *tid == context.tenant_id {
                        Some(office.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            office: &Office,
        ) -> BranchRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, office.id), office.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            office: &Office,
        ) -> BranchRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, office.id)) {
                store.insert(Self::key(context.tenant_id, office.id), office.clone());
                Ok(())
            } else {
                Err(OfficeError::NotFound(office.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: OfficeId,
        ) -> BranchRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(OfficeError::NotFound(id))
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
    async fn in_memory_repository_is_tenant_isolated() {
        let repo = InMemoryBranchRepository::new();
        let office = Office::new(
            OfficeId::new(42),
            TenantId::new(1),
            "Main Branch".to_string(),
            fineract_core::money::Currency::USD,
        )
        .unwrap();

        // Same tenant can read.
        let ctx = make_context(1);
        repo.insert(&ctx, &office).await.unwrap();
        let result = repo.find_by_id(&ctx, OfficeId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Main Branch");

        // Different tenant cannot read (IDOR).
        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, OfficeId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
