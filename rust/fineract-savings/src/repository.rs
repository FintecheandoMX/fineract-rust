//! Savings repositories with tenant isolation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::domain::{
    SavingsAccount, SavingsAccountStatus, SavingsAccountType, SavingsError, SavingsTransaction,
    TransactionType,
};
use fineract_core::ids::SavingsAccountId;
use fineract_core::ids::SavingsTransactionId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type SavingsRepositoryResult<T> = Result<T, SavingsError>;

#[async_trait]
pub trait SavingsAccountRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsRepositoryResult<Option<SavingsAccount>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsRepositoryResult<Vec<SavingsAccount>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        account: &SavingsAccount,
    ) -> SavingsRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        account: &SavingsAccount,
    ) -> SavingsRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsRepositoryResult<()>;
}

#[async_trait]
pub trait SavingsTransactionRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: SavingsTransactionId,
    ) -> SavingsRepositoryResult<Option<SavingsTransaction>>;
    async fn find_all_by_account(
        &self,
        context: &AuthorizedContext,
        account_id: SavingsAccountId,
    ) -> SavingsRepositoryResult<Vec<SavingsTransaction>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        tx: &SavingsTransaction,
    ) -> SavingsRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        tx: &SavingsTransaction,
    ) -> SavingsRepositoryResult<()>;
}

pub struct SqlxSavingsAccountRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxSavingsAccountRepository {
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
    ) -> SavingsRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| SavingsError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl SavingsAccountRepository for SqlxSavingsAccountRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsRepositoryResult<Option<SavingsAccount>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, account_no, account_type, status, currency, balance, interest_rate, opened_date, closed_date, active FROM savings_accounts WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| SavingsError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_account(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsRepositoryResult<Vec<SavingsAccount>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, account_no, account_type, status, currency, balance, interest_rate, opened_date, closed_date, active FROM savings_accounts WHERE tenant_id = $1 ORDER BY account_no",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| SavingsError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_account(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        account: &SavingsAccount,
    ) -> SavingsRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO savings_accounts (id, tenant_id, account_no, account_type, status, currency, balance, interest_rate, opened_date, closed_date, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(account.id.value())
        .bind(account.tenant_id.value())
        .bind(&account.account_no)
        .bind(format!("{:?}", account.account_type))
        .bind(format!("{:?}", account.status))
        .bind(account.currency.code())
        .bind(account.balance.to_string())
        .bind(account.interest_rate.map(|v| v.to_string()))
        .bind(account.opened_date)
        .bind(account.closed_date)
        .bind(account.active)
        .execute(&pool)
        .await
        .map_err(|e| SavingsError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        account: &SavingsAccount,
    ) -> SavingsRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE savings_accounts SET account_no = $2, account_type = $3, status = $4, currency = $5, balance = $6, interest_rate = $7, opened_date = $8, closed_date = $9, active = $10 WHERE id = $1 AND tenant_id = $11",
        )
        .bind(account.id.value())
        .bind(&account.account_no)
        .bind(format!("{:?}", account.account_type))
        .bind(format!("{:?}", account.status))
        .bind(account.currency.code())
        .bind(account.balance.to_string())
        .bind(account.interest_rate.map(|v| v.to_string()))
        .bind(account.opened_date)
        .bind(account.closed_date)
        .bind(account.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| SavingsError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(SavingsError::NotFound(account.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM savings_accounts WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| SavingsError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(SavingsError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_account(row: &sqlx::postgres::PgRow) -> SavingsRepositoryResult<SavingsAccount> {
    Ok(SavingsAccount {
        id: SavingsAccountId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            SavingsError::QueryError {
                detail: "invalid account id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            SavingsError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        account_no: row.try_get("account_no")?,
        account_type: match row.try_get::<String, _>("account_type")?.as_str() {
            "Demand" => SavingsAccountType::Demand,
            "Savings" => SavingsAccountType::Savings,
            "FixedDeposit" => SavingsAccountType::FixedDeposit,
            _ => {
                return Err(SavingsError::QueryError {
                    detail: "invalid account type".to_string(),
                })
            }
        },
        status: match row.try_get::<String, _>("status")?.as_str() {
            "Active" => SavingsAccountStatus::Active,
            "Frozen" => SavingsAccountStatus::Frozen,
            "Closed" => SavingsAccountStatus::Closed,
            "PendingApproval" => SavingsAccountStatus::PendingApproval,
            _ => {
                return Err(SavingsError::QueryError {
                    detail: "invalid status".to_string(),
                })
            }
        },
        currency: fineract_core::money::Currency::from_code(&row.try_get::<String, _>("currency")?)
            .map_err(|_| SavingsError::QueryError {
                detail: "invalid currency".to_string(),
            })?,
        balance: rust_decimal::Decimal::from_str_exact(&row.try_get::<String, _>("balance")?)
            .map_err(|_| SavingsError::QueryError {
                detail: "invalid balance".to_string(),
            })?,
        interest_rate: row
            .try_get::<Option<String>, _>("interest_rate")?
            .map(|v| rust_decimal::Decimal::from_str_exact(&v))
            .transpose()
            .map_err(|_| SavingsError::QueryError {
                detail: "invalid interest rate".to_string(),
            })?,
        opened_date: row.try_get("opened_date")?,
        closed_date: row.try_get("closed_date")?,
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

    pub struct InMemorySavingsAccountRepository {
        store: Mutex<HashMap<(TenantId, SavingsAccountId), SavingsAccount>>,
    }

    impl Default for InMemorySavingsAccountRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemorySavingsAccountRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: SavingsAccountId) -> (TenantId, SavingsAccountId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl SavingsAccountRepository for InMemorySavingsAccountRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: SavingsAccountId,
        ) -> SavingsRepositoryResult<Option<SavingsAccount>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> SavingsRepositoryResult<Vec<SavingsAccount>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), a)| {
                    if *tid == context.tenant_id {
                        Some(a.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            account: &SavingsAccount,
        ) -> SavingsRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, account.id), account.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            account: &SavingsAccount,
        ) -> SavingsRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, account.id)) {
                store.insert(Self::key(context.tenant_id, account.id), account.clone());
                Ok(())
            } else {
                Err(SavingsError::NotFound(account.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: SavingsAccountId,
        ) -> SavingsRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(SavingsError::NotFound(id))
            }
        }
    }

    pub struct InMemorySavingsTransactionRepository {
        store: Mutex<HashMap<(TenantId, SavingsTransactionId), SavingsTransaction>>,
    }

    impl Default for InMemorySavingsTransactionRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemorySavingsTransactionRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: SavingsTransactionId) -> (TenantId, SavingsTransactionId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl SavingsTransactionRepository for InMemorySavingsTransactionRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: SavingsTransactionId,
        ) -> SavingsRepositoryResult<Option<SavingsTransaction>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_account(
            &self,
            context: &AuthorizedContext,
            _account_id: SavingsAccountId,
        ) -> SavingsRepositoryResult<Vec<SavingsTransaction>> {
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

        async fn insert(
            &self,
            context: &AuthorizedContext,
            tx: &SavingsTransaction,
        ) -> SavingsRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, tx.id), tx.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            tx: &SavingsTransaction,
        ) -> SavingsRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, tx.id)) {
                store.insert(Self::key(context.tenant_id, tx.id), tx.clone());
                Ok(())
            } else {
                Err(SavingsError::NotFound(SavingsAccountId::new(0)))
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
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn savings_account_repository_is_tenant_isolated() {
        let repo = InMemorySavingsAccountRepository::new();
        let account = SavingsAccount::new(
            SavingsAccountId::new(42),
            TenantId::new(1),
            "SA-001".to_string(),
            SavingsAccountType::Savings,
            Currency::USD,
            Utc::now(),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &account).await.unwrap();
        let result = repo
            .find_by_id(&ctx, SavingsAccountId::new(42))
            .await
            .unwrap();
        assert_eq!(result.unwrap().account_no, "SA-001");

        let ctx2 = make_context(2);
        let result = repo
            .find_by_id(&ctx2, SavingsAccountId::new(42))
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn savings_transaction_repository_is_tenant_isolated() {
        let repo = InMemorySavingsTransactionRepository::new();
        let tx = SavingsTransaction {
            id: SavingsTransactionId::new(42),
            savings_account_id: SavingsAccountId::new(1),
            tenant_id: TenantId::new(1),
            amount: rust_decimal::Decimal::new(1000, 2),
            transaction_type: TransactionType::Deposit,
            transaction_date: Utc::now(),
            reference: Some("DEP-001".to_string()),
            active: true,
        };

        let ctx = make_context(1);
        repo.insert(&ctx, &tx).await.unwrap();
        let result = repo
            .find_by_id(&ctx, SavingsTransactionId::new(42))
            .await
            .unwrap();
        assert_eq!(result.unwrap().reference, Some("DEP-001".to_string()));

        let ctx2 = make_context(2);
        let result = repo
            .find_by_id(&ctx2, SavingsTransactionId::new(42))
            .await
            .unwrap();
        assert_eq!(result, None);
    }
}
