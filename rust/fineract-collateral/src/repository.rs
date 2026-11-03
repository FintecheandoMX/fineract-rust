//! Collateral repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Collateral, CollateralError, CollateralType};
use fineract_core::ids::CollateralId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type CollateralRepositoryResult<T> = Result<T, CollateralError>;

#[async_trait]
pub trait CollateralRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralRepositoryResult<Option<Collateral>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> CollateralRepositoryResult<Vec<Collateral>>;
    async fn find_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> CollateralRepositoryResult<Vec<Collateral>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        collateral: &Collateral,
    ) -> CollateralRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        collateral: &Collateral,
    ) -> CollateralRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralRepositoryResult<()>;
}

pub struct SqlxCollateralRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxCollateralRepository {
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
    ) -> CollateralRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| CollateralError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl CollateralRepository for SqlxCollateralRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralRepositoryResult<Option<Collateral>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, loan_id, collateral_type, description, value, owned_by, active, created_at, updated_at FROM collateral WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| CollateralError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_collateral(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> CollateralRepositoryResult<Vec<Collateral>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, loan_id, collateral_type, description, value, owned_by, active, created_at, updated_at FROM collateral WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| CollateralError::QueryError { detail: e.to_string() })?;

        rows.into_iter()
            .map(|r| map_row_to_collateral(&r))
            .collect()
    }

    async fn find_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> CollateralRepositoryResult<Vec<Collateral>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, loan_id, collateral_type, description, value, owned_by, active, created_at, updated_at FROM collateral WHERE tenant_id = $1 AND loan_id = $2 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .bind(loan_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| CollateralError::QueryError { detail: e.to_string() })?;

        rows.into_iter()
            .map(|r| map_row_to_collateral(&r))
            .collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        collateral: &Collateral,
    ) -> CollateralRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let collateral_type_str = match collateral.collateral_type {
            CollateralType::Property => "property",
            CollateralType::Vehicle => "vehicle",
            CollateralType::Equipment => "equipment",
            CollateralType::Inventory => "inventory",
            CollateralType::Receivables => "receivables",
            CollateralType::CashDeposit => "cash_deposit",
            CollateralType::Guarantor => "guarantor",
            CollateralType::Other => "other",
        };
        sqlx::query(
            "INSERT INTO collateral (id, tenant_id, loan_id, collateral_type, description, value, owned_by, active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(collateral.id.value())
        .bind(collateral.tenant_id.value())
        .bind(collateral.loan_id.value())
        .bind(collateral_type_str)
        .bind(&collateral.description)
        .bind(&collateral.value)
        .bind(&collateral.owned_by)
        .bind(collateral.active)
        .bind(collateral.created_at)
        .bind(collateral.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| CollateralError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        collateral: &Collateral,
    ) -> CollateralRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let collateral_type_str = match collateral.collateral_type {
            CollateralType::Property => "property",
            CollateralType::Vehicle => "vehicle",
            CollateralType::Equipment => "equipment",
            CollateralType::Inventory => "inventory",
            CollateralType::Receivables => "receivables",
            CollateralType::CashDeposit => "cash_deposit",
            CollateralType::Guarantor => "guarantor",
            CollateralType::Other => "other",
        };
        let result = sqlx::query(
            "UPDATE collateral SET loan_id = $2, collateral_type = $3, description = $4, value = $5, owned_by = $6, active = $7, updated_at = $8 WHERE id = $1 AND tenant_id = $9",
        )
        .bind(collateral.id.value())
        .bind(collateral.loan_id.value())
        .bind(collateral_type_str)
        .bind(&collateral.description)
        .bind(&collateral.value)
        .bind(&collateral.owned_by)
        .bind(collateral.active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| CollateralError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(CollateralError::NotFound(collateral.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM collateral WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| CollateralError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(CollateralError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_collateral(row: &sqlx::postgres::PgRow) -> CollateralRepositoryResult<Collateral> {
    let collateral_type_str: String = row.try_get("collateral_type")?;
    let collateral_type = match collateral_type_str.as_str() {
        "property" => CollateralType::Property,
        "vehicle" => CollateralType::Vehicle,
        "equipment" => CollateralType::Equipment,
        "inventory" => CollateralType::Inventory,
        "receivables" => CollateralType::Receivables,
        "cash_deposit" => CollateralType::CashDeposit,
        "guarantor" => CollateralType::Guarantor,
        "other" => CollateralType::Other,
        _ => {
            return Err(CollateralError::QueryError {
                detail: format!("invalid collateral_type in database: {collateral_type_str}"),
            });
        }
    };

    Ok(Collateral {
        id: CollateralId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            CollateralError::QueryError {
                detail: "invalid collateral id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            CollateralError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        loan_id: LoanId::try_from(row.try_get::<i64, _>("loan_id")?).map_err(|_| {
            CollateralError::QueryError {
                detail: "invalid loan id in database".to_string(),
            }
        })?,
        collateral_type,
        description: row.try_get("description")?,
        value: row.try_get("value")?,
        owned_by: row.try_get("owned_by")?,
        active: row.try_get("active")?,
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

    pub struct InMemoryCollateralRepository {
        store: Mutex<HashMap<(TenantId, CollateralId), Collateral>>,
    }

    impl Default for InMemoryCollateralRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryCollateralRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: CollateralId) -> (TenantId, CollateralId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl CollateralRepository for InMemoryCollateralRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: CollateralId,
        ) -> CollateralRepositoryResult<Option<Collateral>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> CollateralRepositoryResult<Vec<Collateral>> {
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

        async fn find_by_loan_id(
            &self,
            context: &AuthorizedContext,
            loan_id: LoanId,
        ) -> CollateralRepositoryResult<Vec<Collateral>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), c)| {
                    if *tid == context.tenant_id && c.loan_id == loan_id {
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
            collateral: &Collateral,
        ) -> CollateralRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(
                Self::key(context.tenant_id, collateral.id),
                collateral.clone(),
            );
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            collateral: &Collateral,
        ) -> CollateralRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, collateral.id)) {
                store.insert(
                    Self::key(context.tenant_id, collateral.id),
                    collateral.clone(),
                );
                Ok(())
            } else {
                Err(CollateralError::NotFound(collateral.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: CollateralId,
        ) -> CollateralRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(CollateralError::NotFound(id))
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
    async fn collateral_repository_is_tenant_isolated() {
        let repo = InMemoryCollateralRepository::new();
        let collateral = Collateral::new(
            CollateralId::new(42),
            TenantId::new(1),
            LoanId::new(100),
            CollateralType::Property,
            None,
            "100000".to_string(),
            None,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &collateral).await.unwrap();
        let result = repo.find_by_id(&ctx, CollateralId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().value, "100000");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, CollateralId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn collateral_repository_find_by_loan_id() {
        let repo = InMemoryCollateralRepository::new();
        let collateral1 = Collateral::new(
            CollateralId::new(1),
            TenantId::new(1),
            LoanId::new(100),
            CollateralType::Property,
            None,
            "100000".to_string(),
            None,
        )
        .unwrap();
        let collateral2 = Collateral::new(
            CollateralId::new(2),
            TenantId::new(1),
            LoanId::new(200),
            CollateralType::Vehicle,
            None,
            "25000".to_string(),
            None,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &collateral1).await.unwrap();
        repo.insert(&ctx, &collateral2).await.unwrap();

        let loan100_collateral = repo.find_by_loan_id(&ctx, LoanId::new(100)).await.unwrap();
        assert_eq!(loan100_collateral.len(), 1);
        assert_eq!(loan100_collateral[0].value, "100000");

        let loan200_collateral = repo.find_by_loan_id(&ctx, LoanId::new(200)).await.unwrap();
        assert_eq!(loan200_collateral.len(), 1);
        assert_eq!(loan200_collateral[0].value, "25000");
    }
}
