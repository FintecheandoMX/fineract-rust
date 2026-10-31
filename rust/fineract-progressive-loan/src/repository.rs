//! Progressive loan repositories with tenant isolation.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{ProgressiveLoan, ProgressiveLoanError};
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type ProgressiveLoanRepositoryResult<T> = Result<T, ProgressiveLoanError>;

#[async_trait]
pub trait ProgressiveLoanRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanRepositoryResult<Option<ProgressiveLoan>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ProgressiveLoanRepositoryResult<Vec<ProgressiveLoan>>;
    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> ProgressiveLoanRepositoryResult<Vec<ProgressiveLoan>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        loan: &ProgressiveLoan,
    ) -> ProgressiveLoanRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        loan: &ProgressiveLoan,
    ) -> ProgressiveLoanRepositoryResult<()>;
    async fn update_phase(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        phase: crate::domain::ProgressiveLoanPhase,
    ) -> ProgressiveLoanRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanRepositoryResult<()>;
}

pub struct SqlxProgressiveLoanRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxProgressiveLoanRepository {
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
    ) -> ProgressiveLoanRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| ProgressiveLoanError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl ProgressiveLoanRepository for SqlxProgressiveLoanRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanRepositoryResult<Option<ProgressiveLoan>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, client_id, total_amount, currency, current_phase, disbursed_amount, remaining_amount, interest_rate, active FROM progressive_loans WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| ProgressiveLoanError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_loan(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ProgressiveLoanRepositoryResult<Vec<ProgressiveLoan>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, client_id, total_amount, currency, current_phase, disbursed_amount, remaining_amount, interest_rate, active FROM progressive_loans WHERE tenant_id = $1 ORDER BY id",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| ProgressiveLoanError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_loan(&r)).collect()
    }

    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> ProgressiveLoanRepositoryResult<Vec<ProgressiveLoan>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, client_id, total_amount, currency, current_phase, disbursed_amount, remaining_amount, interest_rate, active FROM progressive_loans WHERE tenant_id = $1 AND client_id = $2 ORDER BY id",
        )
        .bind(context.tenant_id.value())
        .bind(client_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| ProgressiveLoanError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_loan(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        loan: &ProgressiveLoan,
    ) -> ProgressiveLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO progressive_loans (id, tenant_id, client_id, total_amount, currency, current_phase, disbursed_amount, remaining_amount, interest_rate, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(loan.id.value())
        .bind(loan.tenant_id.value())
        .bind(loan.client_id.value())
        .bind(loan.total_amount.to_string())
        .bind(loan.currency.code())
        .bind(format!("{:?}", loan.current_phase))
        .bind(loan.disbursed_amount.to_string())
        .bind(loan.remaining_amount.to_string())
        .bind(loan.interest_rate.to_string())
        .bind(loan.active)
        .execute(&pool)
        .await
        .map_err(|e| ProgressiveLoanError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        loan: &ProgressiveLoan,
    ) -> ProgressiveLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE progressive_loans SET client_id = $2, total_amount = $3, currency = $4, current_phase = $5, disbursed_amount = $6, remaining_amount = $7, interest_rate = $8, active = $9 WHERE id = $1 AND tenant_id = $10",
        )
        .bind(loan.id.value())
        .bind(loan.client_id.value())
        .bind(loan.total_amount.to_string())
        .bind(loan.currency.code())
        .bind(format!("{:?}", loan.current_phase))
        .bind(loan.disbursed_amount.to_string())
        .bind(loan.remaining_amount.to_string())
        .bind(loan.interest_rate.to_string())
        .bind(loan.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| ProgressiveLoanError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(ProgressiveLoanError::NotFound(loan.id));
        }
        Ok(())
    }

    async fn update_phase(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        phase: crate::domain::ProgressiveLoanPhase,
    ) -> ProgressiveLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE progressive_loans SET current_phase = $2 WHERE id = $1 AND tenant_id = $3",
        )
        .bind(id.value())
        .bind(format!("{:?}", phase))
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| ProgressiveLoanError::QueryError {
            detail: e.to_string(),
        })?;
        if result.rows_affected() == 0 {
            return Err(ProgressiveLoanError::NotFound(id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM progressive_loans WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| ProgressiveLoanError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(ProgressiveLoanError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_loan(
    row: &sqlx::postgres::PgRow,
) -> ProgressiveLoanRepositoryResult<ProgressiveLoan> {
    Ok(ProgressiveLoan {
        id: LoanId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            ProgressiveLoanError::QueryError {
                detail: "invalid loan id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            ProgressiveLoanError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        client_id: ClientId::try_from(row.try_get::<i64, _>("client_id")?).map_err(|_| {
            ProgressiveLoanError::QueryError {
                detail: "invalid client id".to_string(),
            }
        })?,
        total_amount: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("total_amount")?,
        )
        .map_err(|_| ProgressiveLoanError::QueryError {
            detail: "invalid total amount".to_string(),
        })?,
        currency: fineract_core::money::Currency::from_code(&row.try_get::<String, _>("currency")?)
            .map_err(|_| ProgressiveLoanError::QueryError {
                detail: "invalid currency".to_string(),
            })?,
        current_phase: match row.try_get::<String, _>("current_phase")?.as_str() {
            "Application" => crate::domain::ProgressiveLoanPhase::Application,
            "Approved" => crate::domain::ProgressiveLoanPhase::Approved,
            "Disbursed" => crate::domain::ProgressiveLoanPhase::Disbursed,
            "Completed" => crate::domain::ProgressiveLoanPhase::Completed,
            _ => {
                return Err(ProgressiveLoanError::QueryError {
                    detail: "invalid phase".to_string(),
                })
            }
        },
        disbursed_amount: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("disbursed_amount")?,
        )
        .map_err(|_| ProgressiveLoanError::QueryError {
            detail: "invalid disbursed amount".to_string(),
        })?,
        remaining_amount: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("remaining_amount")?,
        )
        .map_err(|_| ProgressiveLoanError::QueryError {
            detail: "invalid remaining amount".to_string(),
        })?,
        interest_rate: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("interest_rate")?,
        )
        .map_err(|_| ProgressiveLoanError::QueryError {
            detail: "invalid interest rate".to_string(),
        })?,
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

    pub struct InMemoryProgressiveLoanRepository {
        store: Mutex<HashMap<(TenantId, LoanId), ProgressiveLoan>>,
    }

    impl Default for InMemoryProgressiveLoanRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryProgressiveLoanRepository {
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
    impl ProgressiveLoanRepository for InMemoryProgressiveLoanRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> ProgressiveLoanRepositoryResult<Option<ProgressiveLoan>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> ProgressiveLoanRepositoryResult<Vec<ProgressiveLoan>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), l)| {
                    if *tid == context.tenant_id {
                        Some(l.clone())
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
        ) -> ProgressiveLoanRepositoryResult<Vec<ProgressiveLoan>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), l)| {
                    if *tid == context.tenant_id && l.client_id == client_id {
                        Some(l.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            loan: &ProgressiveLoan,
        ) -> ProgressiveLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, loan.id), loan.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            loan: &ProgressiveLoan,
        ) -> ProgressiveLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, loan.id)) {
                store.insert(Self::key(context.tenant_id, loan.id), loan.clone());
                Ok(())
            } else {
                Err(ProgressiveLoanError::NotFound(loan.id))
            }
        }

        async fn update_phase(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
            phase: crate::domain::ProgressiveLoanPhase,
        ) -> ProgressiveLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(loan) = store.get_mut(&Self::key(context.tenant_id, id)) {
                loan.current_phase = phase;
                Ok(())
            } else {
                Err(ProgressiveLoanError::NotFound(id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> ProgressiveLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(ProgressiveLoanError::NotFound(id))
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
    async fn progressive_loan_repository_is_tenant_isolated() {
        let repo = InMemoryProgressiveLoanRepository::new();
        let loan = ProgressiveLoan::new(
            LoanId::new(42),
            TenantId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::new(500000, 2),
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &loan).await.unwrap();
        let result = repo.find_by_id(&ctx, LoanId::new(42)).await.unwrap();
        assert_eq!(
            result.unwrap().total_amount,
            rust_decimal::Decimal::new(500000, 2)
        );

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, LoanId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
