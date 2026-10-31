//! Working capital loan repositories with tenant isolation.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{WorkingCapitalLoan, WorkingCapitalLoanError};
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type WorkingCapitalLoanRepositoryResult<T> = Result<T, WorkingCapitalLoanError>;

#[async_trait]
pub trait WorkingCapitalLoanRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanRepositoryResult<Option<WorkingCapitalLoan>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> WorkingCapitalLoanRepositoryResult<Vec<WorkingCapitalLoan>>;
    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> WorkingCapitalLoanRepositoryResult<Vec<WorkingCapitalLoan>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        loan: &WorkingCapitalLoan,
    ) -> WorkingCapitalLoanRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        loan: &WorkingCapitalLoan,
    ) -> WorkingCapitalLoanRepositoryResult<()>;
    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: crate::domain::WorkingCapitalLoanStatus,
    ) -> WorkingCapitalLoanRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanRepositoryResult<()>;
}

pub struct SqlxWorkingCapitalLoanRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxWorkingCapitalLoanRepository {
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
    ) -> WorkingCapitalLoanRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| WorkingCapitalLoanError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl WorkingCapitalLoanRepository for SqlxWorkingCapitalLoanRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanRepositoryResult<Option<WorkingCapitalLoan>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, client_id, credit_limit, currency, outstanding_balance, available_balance, interest_rate, status, active FROM working_capital_loans WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| WorkingCapitalLoanError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_loan(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> WorkingCapitalLoanRepositoryResult<Vec<WorkingCapitalLoan>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, client_id, credit_limit, currency, outstanding_balance, available_balance, interest_rate, status, active FROM working_capital_loans WHERE tenant_id = $1 ORDER BY id",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| WorkingCapitalLoanError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_loan(&r)).collect()
    }

    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> WorkingCapitalLoanRepositoryResult<Vec<WorkingCapitalLoan>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, client_id, credit_limit, currency, outstanding_balance, available_balance, interest_rate, status, active FROM working_capital_loans WHERE tenant_id = $1 AND client_id = $2 ORDER BY id",
        )
        .bind(context.tenant_id.value())
        .bind(client_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| WorkingCapitalLoanError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_loan(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        loan: &WorkingCapitalLoan,
    ) -> WorkingCapitalLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO working_capital_loans (id, tenant_id, client_id, credit_limit, currency, outstanding_balance, available_balance, interest_rate, status, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(loan.id.value())
        .bind(loan.tenant_id.value())
        .bind(loan.client_id.value())
        .bind(loan.credit_limit.to_string())
        .bind(loan.currency.code())
        .bind(loan.outstanding_balance.to_string())
        .bind(loan.available_balance.to_string())
        .bind(loan.interest_rate.to_string())
        .bind(format!("{:?}", loan.status))
        .bind(loan.active)
        .execute(&pool)
        .await
        .map_err(|e| WorkingCapitalLoanError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        loan: &WorkingCapitalLoan,
    ) -> WorkingCapitalLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE working_capital_loans SET client_id = $2, credit_limit = $3, currency = $4, outstanding_balance = $5, available_balance = $6, interest_rate = $7, status = $8, active = $9 WHERE id = $1 AND tenant_id = $10",
        )
        .bind(loan.id.value())
        .bind(loan.client_id.value())
        .bind(loan.credit_limit.to_string())
        .bind(loan.currency.code())
        .bind(loan.outstanding_balance.to_string())
        .bind(loan.available_balance.to_string())
        .bind(loan.interest_rate.to_string())
        .bind(format!("{:?}", loan.status))
        .bind(loan.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| WorkingCapitalLoanError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(WorkingCapitalLoanError::NotFound(loan.id));
        }
        Ok(())
    }

    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: crate::domain::WorkingCapitalLoanStatus,
    ) -> WorkingCapitalLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE working_capital_loans SET status = $2 WHERE id = $1 AND tenant_id = $3",
        )
        .bind(id.value())
        .bind(format!("{:?}", status))
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| WorkingCapitalLoanError::QueryError {
            detail: e.to_string(),
        })?;
        if result.rows_affected() == 0 {
            return Err(WorkingCapitalLoanError::NotFound(id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result =
            sqlx::query("DELETE FROM working_capital_loans WHERE id = $1 AND tenant_id = $2")
                .bind(id.value())
                .bind(context.tenant_id.value())
                .execute(&pool)
                .await
                .map_err(|e| WorkingCapitalLoanError::QueryError {
                    detail: e.to_string(),
                })?;
        if result.rows_affected() == 0 {
            return Err(WorkingCapitalLoanError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_loan(
    row: &sqlx::postgres::PgRow,
) -> WorkingCapitalLoanRepositoryResult<WorkingCapitalLoan> {
    Ok(WorkingCapitalLoan {
        id: LoanId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            WorkingCapitalLoanError::QueryError {
                detail: "invalid loan id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            WorkingCapitalLoanError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        client_id: ClientId::try_from(row.try_get::<i64, _>("client_id")?).map_err(|_| {
            WorkingCapitalLoanError::QueryError {
                detail: "invalid client id".to_string(),
            }
        })?,
        credit_limit: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("credit_limit")?,
        )
        .map_err(|_| WorkingCapitalLoanError::QueryError {
            detail: "invalid credit limit".to_string(),
        })?,
        currency: fineract_core::money::Currency::from_code(&row.try_get::<String, _>("currency")?)
            .map_err(|_| WorkingCapitalLoanError::QueryError {
                detail: "invalid currency".to_string(),
            })?,
        outstanding_balance: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("outstanding_balance")?,
        )
        .map_err(|_| WorkingCapitalLoanError::QueryError {
            detail: "invalid outstanding balance".to_string(),
        })?,
        available_balance: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("available_balance")?,
        )
        .map_err(|_| WorkingCapitalLoanError::QueryError {
            detail: "invalid available balance".to_string(),
        })?,
        interest_rate: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("interest_rate")?,
        )
        .map_err(|_| WorkingCapitalLoanError::QueryError {
            detail: "invalid interest rate".to_string(),
        })?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "Active" => crate::domain::WorkingCapitalLoanStatus::Active,
            "Suspended" => crate::domain::WorkingCapitalLoanStatus::Suspended,
            "Closed" => crate::domain::WorkingCapitalLoanStatus::Closed,
            _ => {
                return Err(WorkingCapitalLoanError::QueryError {
                    detail: "invalid status".to_string(),
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
    use fineract_core::money::Currency;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryWorkingCapitalLoanRepository {
        store: Mutex<HashMap<(TenantId, LoanId), WorkingCapitalLoan>>,
    }

    impl Default for InMemoryWorkingCapitalLoanRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryWorkingCapitalLoanRepository {
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
    impl WorkingCapitalLoanRepository for InMemoryWorkingCapitalLoanRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> WorkingCapitalLoanRepositoryResult<Option<WorkingCapitalLoan>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> WorkingCapitalLoanRepositoryResult<Vec<WorkingCapitalLoan>> {
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
        ) -> WorkingCapitalLoanRepositoryResult<Vec<WorkingCapitalLoan>> {
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
            loan: &WorkingCapitalLoan,
        ) -> WorkingCapitalLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, loan.id), loan.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            loan: &WorkingCapitalLoan,
        ) -> WorkingCapitalLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, loan.id)) {
                store.insert(Self::key(context.tenant_id, loan.id), loan.clone());
                Ok(())
            } else {
                Err(WorkingCapitalLoanError::NotFound(loan.id))
            }
        }

        async fn update_status(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
            status: crate::domain::WorkingCapitalLoanStatus,
        ) -> WorkingCapitalLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(loan) = store.get_mut(&Self::key(context.tenant_id, id)) {
                loan.status = status;
                Ok(())
            } else {
                Err(WorkingCapitalLoanError::NotFound(id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> WorkingCapitalLoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(WorkingCapitalLoanError::NotFound(id))
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
    async fn working_capital_loan_repository_is_tenant_isolated() {
        let repo = InMemoryWorkingCapitalLoanRepository::new();
        let loan = WorkingCapitalLoan::new(
            LoanId::new(42),
            TenantId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::new(200000, 2),
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &loan).await.unwrap();
        let result = repo.find_by_id(&ctx, LoanId::new(42)).await.unwrap();
        assert_eq!(
            result.unwrap().credit_limit,
            rust_decimal::Decimal::new(200000, 2)
        );

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, LoanId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
