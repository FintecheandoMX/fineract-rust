//! Investor repository.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Investor, InvestorError};
use fineract_core::ids::InvestorId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type InvestorRepositoryResult<T> = Result<T, InvestorError>;

#[async_trait]
pub trait InvestorRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorRepositoryResult<Option<Investor>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> InvestorRepositoryResult<Vec<Investor>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        investor: &Investor,
    ) -> InvestorRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        investor: &Investor,
    ) -> InvestorRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorRepositoryResult<()>;
}

pub struct SqlxInvestorRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxInvestorRepository {
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
    ) -> InvestorRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| InvestorError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl InvestorRepository for SqlxInvestorRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorRepositoryResult<Option<Investor>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, external_id, active FROM investors WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| InvestorError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_investor(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> InvestorRepositoryResult<Vec<Investor>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, external_id, active FROM investors WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| InvestorError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_investor(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        investor: &Investor,
    ) -> InvestorRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO investors (id, tenant_id, name, external_id, active) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(investor.id.value())
        .bind(investor.tenant_id.value())
        .bind(&investor.name)
        .bind(&investor.external_id)
        .bind(investor.active)
        .execute(&pool)
        .await
        .map_err(|e| InvestorError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        investor: &Investor,
    ) -> InvestorRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE investors SET name = $2, external_id = $3, active = $4 WHERE id = $1 AND tenant_id = $5",
        )
        .bind(investor.id.value())
        .bind(&investor.name)
        .bind(&investor.external_id)
        .bind(investor.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| InvestorError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(InvestorError::NotFound(investor.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM investors WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| InvestorError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(InvestorError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_investor(row: &sqlx::postgres::PgRow) -> InvestorRepositoryResult<Investor> {
    Ok(Investor {
        id: InvestorId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            InvestorError::QueryError {
                detail: "invalid investor id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            InvestorError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        external_id: row.try_get("external_id")?,
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

    pub struct InMemoryInvestorRepository {
        store: Mutex<HashMap<(TenantId, InvestorId), Investor>>,
    }

    impl Default for InMemoryInvestorRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryInvestorRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: InvestorId) -> (TenantId, InvestorId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl InvestorRepository for InMemoryInvestorRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: InvestorId,
        ) -> InvestorRepositoryResult<Option<Investor>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> InvestorRepositoryResult<Vec<Investor>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), inv)| {
                    if *tid == context.tenant_id {
                        Some(inv.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            investor: &Investor,
        ) -> InvestorRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, investor.id), investor.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            investor: &Investor,
        ) -> InvestorRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, investor.id)) {
                store.insert(Self::key(context.tenant_id, investor.id), investor.clone());
                Ok(())
            } else {
                Err(InvestorError::NotFound(investor.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: InvestorId,
        ) -> InvestorRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(InvestorError::NotFound(id))
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
    async fn investor_repository_is_tenant_isolated() {
        let repo = InMemoryInvestorRepository::new();
        let inv = Investor::new(
            InvestorId::new(42),
            TenantId::new(1),
            "Capital Partners".to_string(),
            Some("EXT-001".to_string()),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &inv).await.unwrap();
        let result = repo.find_by_id(&ctx, InvestorId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Capital Partners");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, InvestorId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
