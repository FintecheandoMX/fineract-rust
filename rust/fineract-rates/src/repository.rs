//! Rates repository.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Rate, RateError, RatePeriod};
use fineract_core::ids::RateId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type RateRepositoryResult<T> = Result<T, RateError>;

#[async_trait]
pub trait RateRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: RateId,
    ) -> RateRepositoryResult<Option<Rate>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> RateRepositoryResult<Vec<Rate>>;
    async fn insert(&self, context: &AuthorizedContext, rate: &Rate) -> RateRepositoryResult<()>;
    async fn update(&self, context: &AuthorizedContext, rate: &Rate) -> RateRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: RateId) -> RateRepositoryResult<()>;
}

pub struct SqlxRateRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxRateRepository {
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
    ) -> RateRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| RateError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl RateRepository for SqlxRateRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: RateId,
    ) -> RateRepositoryResult<Option<Rate>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, period, annual_percentage_rate, active FROM rates WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| RateError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_rate(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> RateRepositoryResult<Vec<Rate>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, period, annual_percentage_rate, active FROM rates WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| RateError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_rate(&r)).collect()
    }

    async fn insert(&self, context: &AuthorizedContext, rate: &Rate) -> RateRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO rates (id, tenant_id, name, period, annual_percentage_rate, active) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(rate.id.value())
        .bind(rate.tenant_id.value())
        .bind(&rate.name)
        .bind(format!("{:?}", rate.period))
        .bind(rate.annual_percentage_rate.to_string())
        .bind(rate.active)
        .execute(&pool)
        .await
        .map_err(|e| RateError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(&self, context: &AuthorizedContext, rate: &Rate) -> RateRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE rates SET name = $2, period = $3, annual_percentage_rate = $4, active = $5 WHERE id = $1 AND tenant_id = $6",
        )
        .bind(rate.id.value())
        .bind(&rate.name)
        .bind(format!("{:?}", rate.period))
        .bind(rate.annual_percentage_rate.to_string())
        .bind(rate.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| RateError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(RateError::NotFound(rate.id));
        }
        Ok(())
    }

    async fn delete(&self, context: &AuthorizedContext, id: RateId) -> RateRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM rates WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| RateError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(RateError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_rate(row: &sqlx::postgres::PgRow) -> RateRepositoryResult<Rate> {
    Ok(Rate {
        id: RateId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| RateError::QueryError {
            detail: "invalid rate id".to_string(),
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            RateError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        period: match row.try_get::<String, _>("period")?.as_str() {
            "Daily" => RatePeriod::Daily,
            "Weekly" => RatePeriod::Weekly,
            "Monthly" => RatePeriod::Monthly,
            "Yearly" => RatePeriod::Yearly,
            _ => {
                return Err(RateError::QueryError {
                    detail: "invalid period".to_string(),
                })
            }
        },
        annual_percentage_rate: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("annual_percentage_rate")?,
        )
        .map_err(|_| RateError::QueryError {
            detail: "invalid apr".to_string(),
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

    pub struct InMemoryRateRepository {
        store: Mutex<HashMap<(TenantId, RateId), Rate>>,
    }

    impl Default for InMemoryRateRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryRateRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: RateId) -> (TenantId, RateId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl RateRepository for InMemoryRateRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: RateId,
        ) -> RateRepositoryResult<Option<Rate>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> RateRepositoryResult<Vec<Rate>> {
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

        async fn insert(
            &self,
            context: &AuthorizedContext,
            rate: &Rate,
        ) -> RateRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, rate.id), rate.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            rate: &Rate,
        ) -> RateRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, rate.id)) {
                store.insert(Self::key(context.tenant_id, rate.id), rate.clone());
                Ok(())
            } else {
                Err(RateError::NotFound(rate.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: RateId,
        ) -> RateRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(RateError::NotFound(id))
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
    async fn rate_repository_is_tenant_isolated() {
        let repo = InMemoryRateRepository::new();
        let rate = Rate::new(
            RateId::new(42),
            TenantId::new(1),
            "Standard".to_string(),
            RatePeriod::Yearly,
            rust_decimal::Decimal::new(500, 2),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &rate).await.unwrap();
        let result = repo.find_by_id(&ctx, RateId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Standard");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, RateId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
