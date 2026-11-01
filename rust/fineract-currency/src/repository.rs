//! Currency repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Currency, CurrencyError};
use fineract_core::ids::CurrencyId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type CurrencyRepositoryResult<T> = Result<T, CurrencyError>;

#[async_trait]
pub trait CurrencyRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyRepositoryResult<Option<Currency>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> CurrencyRepositoryResult<Vec<Currency>>;
    async fn find_by_code(
        &self,
        context: &AuthorizedContext,
        code: &str,
    ) -> CurrencyRepositoryResult<Option<Currency>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        currency: &Currency,
    ) -> CurrencyRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        currency: &Currency,
    ) -> CurrencyRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyRepositoryResult<()>;
}

pub struct SqlxCurrencyRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxCurrencyRepository {
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
    ) -> CurrencyRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| CurrencyError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl CurrencyRepository for SqlxCurrencyRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyRepositoryResult<Option<Currency>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, code, name, display_symbol, display_label, decimal_places, is_active, created_at, updated_at FROM currencies WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| CurrencyError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_currency(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> CurrencyRepositoryResult<Vec<Currency>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, code, name, display_symbol, display_label, decimal_places, is_active, created_at, updated_at FROM currencies WHERE tenant_id = $1 ORDER BY code",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| CurrencyError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_currency(&r)).collect()
    }

    async fn find_by_code(
        &self,
        context: &AuthorizedContext,
        code: &str,
    ) -> CurrencyRepositoryResult<Option<Currency>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, code, name, display_symbol, display_label, decimal_places, is_active, created_at, updated_at FROM currencies WHERE code = $1 AND tenant_id = $2",
        )
        .bind(code)
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| CurrencyError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_currency(&r)).transpose()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        currency: &Currency,
    ) -> CurrencyRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO currencies (id, tenant_id, code, name, display_symbol, display_label, decimal_places, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(currency.id.value())
        .bind(currency.tenant_id.value())
        .bind(&currency.code)
        .bind(&currency.name)
        .bind(&currency.display_symbol)
        .bind(&currency.display_label)
        .bind(currency.decimal_places)
        .bind(currency.is_active)
        .bind(currency.created_at)
        .bind(currency.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| CurrencyError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        currency: &Currency,
    ) -> CurrencyRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE currencies SET code = $2, name = $3, display_symbol = $4, display_label = $5, decimal_places = $6, is_active = $7, updated_at = $8 WHERE id = $1 AND tenant_id = $9",
        )
        .bind(currency.id.value())
        .bind(&currency.code)
        .bind(&currency.name)
        .bind(&currency.display_symbol)
        .bind(&currency.display_label)
        .bind(currency.decimal_places)
        .bind(currency.is_active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| CurrencyError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(CurrencyError::NotFound(currency.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM currencies WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| CurrencyError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(CurrencyError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_currency(row: &sqlx::postgres::PgRow) -> CurrencyRepositoryResult<Currency> {
    Ok(Currency {
        id: CurrencyId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            CurrencyError::QueryError {
                detail: "invalid currency id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            CurrencyError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        display_symbol: row.try_get("display_symbol")?,
        display_label: row.try_get("display_label")?,
        decimal_places: row.try_get("decimal_places")?,
        is_active: row.try_get("is_active")?,
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

    pub struct InMemoryCurrencyRepository {
        store: Mutex<HashMap<(TenantId, CurrencyId), Currency>>,
    }

    impl Default for InMemoryCurrencyRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryCurrencyRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: CurrencyId) -> (TenantId, CurrencyId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl CurrencyRepository for InMemoryCurrencyRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: CurrencyId,
        ) -> CurrencyRepositoryResult<Option<Currency>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> CurrencyRepositoryResult<Vec<Currency>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), c)| {
                    if *tid == context.tenant_id {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_by_code(
            &self,
            context: &AuthorizedContext,
            code: &str,
        ) -> CurrencyRepositoryResult<Option<Currency>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), c)| {
                    if *tid == context.tenant_id && c.code == code {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .next())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            currency: &Currency,
        ) -> CurrencyRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, currency.id), currency.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            currency: &Currency,
        ) -> CurrencyRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, currency.id)) {
                store.insert(Self::key(context.tenant_id, currency.id), currency.clone());
                Ok(())
            } else {
                Err(CurrencyError::NotFound(currency.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: CurrencyId,
        ) -> CurrencyRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(CurrencyError::NotFound(id))
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
    async fn currency_repository_is_tenant_isolated() {
        let repo = InMemoryCurrencyRepository::new();
        let currency = Currency::new(
            CurrencyId::new(42),
            TenantId::new(1),
            "USD".to_string(),
            "US Dollar".to_string(),
            "$".to_string(),
            "USD".to_string(),
            2,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &currency).await.unwrap();
        let result = repo.find_by_id(&ctx, CurrencyId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().code, "USD");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, CurrencyId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
