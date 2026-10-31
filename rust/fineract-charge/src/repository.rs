//! Charge repository.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Charge, ChargeError, ChargeType};
use fineract_core::ids::ChargeId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type ChargeRepositoryResult<T> = Result<T, ChargeError>;

#[async_trait]
pub trait ChargeRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: ChargeId,
    ) -> ChargeRepositoryResult<Option<Charge>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ChargeRepositoryResult<Vec<Charge>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        charge: &Charge,
    ) -> ChargeRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        charge: &Charge,
    ) -> ChargeRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: ChargeId)
        -> ChargeRepositoryResult<()>;
}

pub struct SqlxChargeRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxChargeRepository {
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
    ) -> ChargeRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| ChargeError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl ChargeRepository for SqlxChargeRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: ChargeId,
    ) -> ChargeRepositoryResult<Option<Charge>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, charge_type, value, active FROM charges WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| ChargeError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_charge(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ChargeRepositoryResult<Vec<Charge>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, charge_type, value, active FROM charges WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| ChargeError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_charge(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        charge: &Charge,
    ) -> ChargeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO charges (id, tenant_id, name, charge_type, value, active) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(charge.id.value())
        .bind(charge.tenant_id.value())
        .bind(&charge.name)
        .bind(format!("{:?}", charge.charge_type))
        .bind(charge.value.to_string())
        .bind(charge.active)
        .execute(&pool)
        .await
        .map_err(|e| ChargeError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        charge: &Charge,
    ) -> ChargeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE charges SET name = $2, charge_type = $3, value = $4, active = $5 WHERE id = $1 AND tenant_id = $6",
        )
        .bind(charge.id.value())
        .bind(&charge.name)
        .bind(format!("{:?}", charge.charge_type))
        .bind(charge.value.to_string())
        .bind(charge.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| ChargeError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(ChargeError::NotFound(charge.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: ChargeId,
    ) -> ChargeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM charges WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| ChargeError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(ChargeError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_charge(row: &sqlx::postgres::PgRow) -> ChargeRepositoryResult<Charge> {
    Ok(Charge {
        id: ChargeId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            ChargeError::QueryError {
                detail: "invalid charge id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            ChargeError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        charge_type: match row.try_get::<String, _>("charge_type")?.as_str() {
            "Fixed" => ChargeType::Fixed,
            "Percentage" => ChargeType::Percentage,
            _ => {
                return Err(ChargeError::QueryError {
                    detail: "invalid charge type".to_string(),
                })
            }
        },
        value: rust_decimal::Decimal::from_str_exact(&row.try_get::<String, _>("value")?).map_err(
            |_| ChargeError::QueryError {
                detail: "invalid charge value".to_string(),
            },
        )?,
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

    pub struct InMemoryChargeRepository {
        store: Mutex<HashMap<(TenantId, ChargeId), Charge>>,
    }

    impl Default for InMemoryChargeRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryChargeRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: ChargeId) -> (TenantId, ChargeId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl ChargeRepository for InMemoryChargeRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: ChargeId,
        ) -> ChargeRepositoryResult<Option<Charge>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> ChargeRepositoryResult<Vec<Charge>> {
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

        async fn insert(
            &self,
            context: &AuthorizedContext,
            charge: &Charge,
        ) -> ChargeRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, charge.id), charge.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            charge: &Charge,
        ) -> ChargeRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, charge.id)) {
                store.insert(Self::key(context.tenant_id, charge.id), charge.clone());
                Ok(())
            } else {
                Err(ChargeError::NotFound(charge.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: ChargeId,
        ) -> ChargeRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(ChargeError::NotFound(id))
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
    async fn charge_repository_is_tenant_isolated() {
        let repo = InMemoryChargeRepository::new();
        let charge = Charge::new(
            ChargeId::new(42),
            TenantId::new(1),
            "Fee".to_string(),
            ChargeType::Fixed,
            rust_decimal::Decimal::new(100, 2),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &charge).await.unwrap();
        let result = repo.find_by_id(&ctx, ChargeId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Fee");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, ChargeId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
