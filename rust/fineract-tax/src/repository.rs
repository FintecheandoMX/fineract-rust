//! Tax repository.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Tax, TaxError, TaxInclusion};
use fineract_core::ids::TaxId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type TaxRepositoryResult<T> = Result<T, TaxError>;

#[async_trait]
pub trait TaxRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: TaxId,
    ) -> TaxRepositoryResult<Option<Tax>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> TaxRepositoryResult<Vec<Tax>>;
    async fn insert(&self, context: &AuthorizedContext, tax: &Tax) -> TaxRepositoryResult<()>;
    async fn update(&self, context: &AuthorizedContext, tax: &Tax) -> TaxRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: TaxId) -> TaxRepositoryResult<()>;
}

pub struct SqlxTaxRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxTaxRepository {
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
    ) -> TaxRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| TaxError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl TaxRepository for SqlxTaxRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: TaxId,
    ) -> TaxRepositoryResult<Option<Tax>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, rate, inclusion, active FROM taxes WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| TaxError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_tax(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> TaxRepositoryResult<Vec<Tax>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, rate, inclusion, active FROM taxes WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| TaxError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_tax(&r)).collect()
    }

    async fn insert(&self, context: &AuthorizedContext, tax: &Tax) -> TaxRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO taxes (id, tenant_id, name, rate, inclusion, active) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(tax.id.value())
        .bind(tax.tenant_id.value())
        .bind(&tax.name)
        .bind(tax.rate.to_string())
        .bind(format!("{:?}", tax.inclusion))
        .bind(tax.active)
        .execute(&pool)
        .await
        .map_err(|e| TaxError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(&self, context: &AuthorizedContext, tax: &Tax) -> TaxRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE taxes SET name = $2, rate = $3, inclusion = $4, active = $5 WHERE id = $1 AND tenant_id = $6",
        )
        .bind(tax.id.value())
        .bind(&tax.name)
        .bind(tax.rate.to_string())
        .bind(format!("{:?}", tax.inclusion))
        .bind(tax.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| TaxError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(TaxError::NotFound(tax.id));
        }
        Ok(())
    }

    async fn delete(&self, context: &AuthorizedContext, id: TaxId) -> TaxRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM taxes WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| TaxError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(TaxError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_tax(row: &sqlx::postgres::PgRow) -> TaxRepositoryResult<Tax> {
    Ok(Tax {
        id: TaxId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| TaxError::QueryError {
            detail: "invalid tax id".to_string(),
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            TaxError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        rate: rust_decimal::Decimal::from_str_exact(&row.try_get::<String, _>("rate")?).map_err(
            |_| TaxError::QueryError {
                detail: "invalid rate".to_string(),
            },
        )?,
        inclusion: match row.try_get::<String, _>("inclusion")?.as_str() {
            "Included" => TaxInclusion::Included,
            "Excluded" => TaxInclusion::Excluded,
            _ => {
                return Err(TaxError::QueryError {
                    detail: "invalid inclusion".to_string(),
                })
            }
        },
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

    pub struct InMemoryTaxRepository {
        store: Mutex<HashMap<(TenantId, TaxId), Tax>>,
    }

    impl Default for InMemoryTaxRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryTaxRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: TaxId) -> (TenantId, TaxId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl TaxRepository for InMemoryTaxRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: TaxId,
        ) -> TaxRepositoryResult<Option<Tax>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> TaxRepositoryResult<Vec<Tax>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), t)| {
                    if *tid == context.tenant_id {
                        Some(t.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(&self, context: &AuthorizedContext, tax: &Tax) -> TaxRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, tax.id), tax.clone());
            Ok(())
        }

        async fn update(&self, context: &AuthorizedContext, tax: &Tax) -> TaxRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, tax.id)) {
                store.insert(Self::key(context.tenant_id, tax.id), tax.clone());
                Ok(())
            } else {
                Err(TaxError::NotFound(tax.id))
            }
        }

        async fn delete(&self, context: &AuthorizedContext, id: TaxId) -> TaxRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(TaxError::NotFound(id))
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
    async fn tax_repository_is_tenant_isolated() {
        let repo = InMemoryTaxRepository::new();
        let tax = Tax::new(
            TaxId::new(42),
            TenantId::new(1),
            "VAT".to_string(),
            rust_decimal::Decimal::new(1000, 2),
            TaxInclusion::Excluded,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &tax).await.unwrap();
        let result = repo.find_by_id(&ctx, TaxId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "VAT");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, TaxId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
