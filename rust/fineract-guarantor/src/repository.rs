//! Guarantor repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Guarantor, GuarantorError};
use fineract_core::ids::GuarantorId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type GuarantorRepositoryResult<T> = Result<T, GuarantorError>;

#[async_trait]
pub trait GuarantorRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorRepositoryResult<Option<Guarantor>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> GuarantorRepositoryResult<Vec<Guarantor>>;
    async fn find_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> GuarantorRepositoryResult<Vec<Guarantor>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        guarantor: &Guarantor,
    ) -> GuarantorRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        guarantor: &Guarantor,
    ) -> GuarantorRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorRepositoryResult<()>;
}

pub struct SqlxGuarantorRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxGuarantorRepository {
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
    ) -> GuarantorRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| GuarantorError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl GuarantorRepository for SqlxGuarantorRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorRepositoryResult<Option<Guarantor>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, loan_id, name, email, phone, address, identification_number, net_worth, active, created_at, updated_at FROM guarantors WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| GuarantorError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_guarantor(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> GuarantorRepositoryResult<Vec<Guarantor>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, loan_id, name, email, phone, address, identification_number, net_worth, active, created_at, updated_at FROM guarantors WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| GuarantorError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_guarantor(&r)).collect()
    }

    async fn find_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> GuarantorRepositoryResult<Vec<Guarantor>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, loan_id, name, email, phone, address, identification_number, net_worth, active, created_at, updated_at FROM guarantors WHERE tenant_id = $1 AND loan_id = $2 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .bind(loan_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| GuarantorError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_guarantor(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        guarantor: &Guarantor,
    ) -> GuarantorRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO guarantors (id, tenant_id, loan_id, name, email, phone, address, identification_number, net_worth, active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(guarantor.id.value())
        .bind(guarantor.tenant_id.value())
        .bind(guarantor.loan_id.value())
        .bind(&guarantor.name)
        .bind(&guarantor.email)
        .bind(&guarantor.phone)
        .bind(&guarantor.address)
        .bind(&guarantor.identification_number)
        .bind(&guarantor.net_worth)
        .bind(guarantor.active)
        .bind(guarantor.created_at)
        .bind(guarantor.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| GuarantorError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        guarantor: &Guarantor,
    ) -> GuarantorRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE guarantors SET loan_id = $2, name = $3, email = $4, phone = $5, address = $6, identification_number = $7, net_worth = $8, active = $9, updated_at = $10 WHERE id = $1 AND tenant_id = $11",
        )
        .bind(guarantor.id.value())
        .bind(guarantor.loan_id.value())
        .bind(&guarantor.name)
        .bind(&guarantor.email)
        .bind(&guarantor.phone)
        .bind(&guarantor.address)
        .bind(&guarantor.identification_number)
        .bind(&guarantor.net_worth)
        .bind(guarantor.active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| GuarantorError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(GuarantorError::NotFound(guarantor.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM guarantors WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| GuarantorError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(GuarantorError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_guarantor(row: &sqlx::postgres::PgRow) -> GuarantorRepositoryResult<Guarantor> {
    Ok(Guarantor {
        id: GuarantorId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            GuarantorError::QueryError {
                detail: "invalid guarantor id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            GuarantorError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        loan_id: LoanId::try_from(row.try_get::<i64, _>("loan_id")?).map_err(|_| {
            GuarantorError::QueryError {
                detail: "invalid loan id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        email: row.try_get("email")?,
        phone: row.try_get("phone")?,
        address: row.try_get("address")?,
        identification_number: row.try_get("identification_number")?,
        net_worth: row.try_get("net_worth")?,
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

    pub struct InMemoryGuarantorRepository {
        store: Mutex<HashMap<(TenantId, GuarantorId), Guarantor>>,
    }

    impl Default for InMemoryGuarantorRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryGuarantorRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: GuarantorId) -> (TenantId, GuarantorId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl GuarantorRepository for InMemoryGuarantorRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: GuarantorId,
        ) -> GuarantorRepositoryResult<Option<Guarantor>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> GuarantorRepositoryResult<Vec<Guarantor>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), g)| {
                    if *tid == context.tenant_id {
                        Some(g.clone())
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
        ) -> GuarantorRepositoryResult<Vec<Guarantor>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), g)| {
                    if *tid == context.tenant_id && g.loan_id == loan_id {
                        Some(g.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            guarantor: &Guarantor,
        ) -> GuarantorRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(
                Self::key(context.tenant_id, guarantor.id),
                guarantor.clone(),
            );
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            guarantor: &Guarantor,
        ) -> GuarantorRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, guarantor.id)) {
                store.insert(
                    Self::key(context.tenant_id, guarantor.id),
                    guarantor.clone(),
                );
                Ok(())
            } else {
                Err(GuarantorError::NotFound(guarantor.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: GuarantorId,
        ) -> GuarantorRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(GuarantorError::NotFound(id))
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
    async fn guarantor_repository_is_tenant_isolated() {
        let repo = InMemoryGuarantorRepository::new();
        let guarantor = Guarantor::new(
            GuarantorId::new(42),
            TenantId::new(1),
            LoanId::new(100),
            "John Doe".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &guarantor).await.unwrap();
        let result = repo.find_by_id(&ctx, GuarantorId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "John Doe");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, GuarantorId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn guarantor_repository_find_by_loan_id() {
        let repo = InMemoryGuarantorRepository::new();
        let guarantor1 = Guarantor::new(
            GuarantorId::new(1),
            TenantId::new(1),
            LoanId::new(100),
            "John Doe".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let guarantor2 = Guarantor::new(
            GuarantorId::new(2),
            TenantId::new(1),
            LoanId::new(200),
            "Jane Smith".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &guarantor1).await.unwrap();
        repo.insert(&ctx, &guarantor2).await.unwrap();

        let loan100_guarantors = repo.find_by_loan_id(&ctx, LoanId::new(100)).await.unwrap();
        assert_eq!(loan100_guarantors.len(), 1);
        assert_eq!(loan100_guarantors[0].name, "John Doe");

        let loan200_guarantors = repo.find_by_loan_id(&ctx, LoanId::new(200)).await.unwrap();
        assert_eq!(loan200_guarantors.len(), 1);
        assert_eq!(loan200_guarantors[0].name, "Jane Smith");
    }
}
