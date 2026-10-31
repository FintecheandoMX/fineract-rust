//! Accounting repositories with tenant isolation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::domain::{
    Account, AccountStatus, AccountType, AccountingError, AccountingPeriod, JournalEntry,
    JournalEntryLine,
};
use fineract_core::ids::AccountId;
use fineract_core::ids::AccountingPeriodId;
use fineract_core::ids::JournalEntryId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type AccountingRepositoryResult<T> = Result<T, AccountingError>;

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingRepositoryResult<Option<Account>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> AccountingRepositoryResult<Vec<Account>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        account: &Account,
    ) -> AccountingRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        account: &Account,
    ) -> AccountingRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingRepositoryResult<()>;
}

#[async_trait]
pub trait JournalEntryRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: JournalEntryId,
    ) -> AccountingRepositoryResult<Option<JournalEntry>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> AccountingRepositoryResult<Vec<JournalEntry>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        entry: &JournalEntry,
    ) -> AccountingRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        entry: &JournalEntry,
    ) -> AccountingRepositoryResult<()>;
}

#[async_trait]
pub trait AccountingPeriodRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: AccountingPeriodId,
    ) -> AccountingRepositoryResult<Option<AccountingPeriod>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> AccountingRepositoryResult<Vec<AccountingPeriod>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        period: &AccountingPeriod,
    ) -> AccountingRepositoryResult<()>;
}

pub struct SqlxAccountRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxAccountRepository {
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
    ) -> AccountingRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| AccountingError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl AccountRepository for SqlxAccountRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingRepositoryResult<Option<Account>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, account_code, account_name, account_type, currency, balance, status, active FROM gl_accounts WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| AccountingError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_account(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> AccountingRepositoryResult<Vec<Account>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, account_code, account_name, account_type, currency, balance, status, active FROM gl_accounts WHERE tenant_id = $1 ORDER BY account_code",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| AccountingError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_account(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        account: &Account,
    ) -> AccountingRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO gl_accounts (id, tenant_id, account_code, account_name, account_type, currency, balance, status, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(account.id.value())
        .bind(account.tenant_id.value())
        .bind(&account.account_code)
        .bind(&account.account_name)
        .bind(format!("{:?}", account.account_type))
        .bind(account.currency.code())
        .bind(account.balance.to_string())
        .bind(format!("{:?}", account.status))
        .bind(account.active)
        .execute(&pool)
        .await
        .map_err(|e| AccountingError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        account: &Account,
    ) -> AccountingRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE gl_accounts SET account_code = $2, account_name = $3, account_type = $4, currency = $5, balance = $6, status = $7, active = $8 WHERE id = $1 AND tenant_id = $9",
        )
        .bind(account.id.value())
        .bind(&account.account_code)
        .bind(&account.account_name)
        .bind(format!("{:?}", account.account_type))
        .bind(account.currency.code())
        .bind(account.balance.to_string())
        .bind(format!("{:?}", account.status))
        .bind(account.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| AccountingError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(AccountingError::NotFound(account.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM gl_accounts WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| AccountingError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(AccountingError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_account(row: &sqlx::postgres::PgRow) -> AccountingRepositoryResult<Account> {
    Ok(Account {
        id: AccountId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            AccountingError::QueryError {
                detail: "invalid account id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            AccountingError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        account_code: row.try_get("account_code")?,
        account_name: row.try_get("account_name")?,
        account_type: match row.try_get::<String, _>("account_type")?.as_str() {
            "Asset" => AccountType::Asset,
            "Liability" => AccountType::Liability,
            "Equity" => AccountType::Equity,
            "Revenue" => AccountType::Revenue,
            "Expense" => AccountType::Expense,
            _ => {
                return Err(AccountingError::QueryError {
                    detail: "invalid account type".to_string(),
                })
            }
        },
        currency: fineract_core::money::Currency::from_code(&row.try_get::<String, _>("currency")?)
            .map_err(|_| AccountingError::QueryError {
                detail: "invalid currency".to_string(),
            })?,
        balance: rust_decimal::Decimal::from_str_exact(&row.try_get::<String, _>("balance")?)
            .map_err(|_| AccountingError::QueryError {
                detail: "invalid balance".to_string(),
            })?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "Active" => AccountStatus::Active,
            "Closed" => AccountStatus::Closed,
            _ => {
                return Err(AccountingError::QueryError {
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

    pub struct InMemoryAccountRepository {
        store: Mutex<HashMap<(TenantId, AccountId), Account>>,
    }

    impl Default for InMemoryAccountRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryAccountRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: AccountId) -> (TenantId, AccountId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl AccountRepository for InMemoryAccountRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: AccountId,
        ) -> AccountingRepositoryResult<Option<Account>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> AccountingRepositoryResult<Vec<Account>> {
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
            account: &Account,
        ) -> AccountingRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, account.id), account.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            account: &Account,
        ) -> AccountingRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, account.id)) {
                store.insert(Self::key(context.tenant_id, account.id), account.clone());
                Ok(())
            } else {
                Err(AccountingError::NotFound(account.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: AccountId,
        ) -> AccountingRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(AccountingError::NotFound(id))
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
    async fn account_repository_is_tenant_isolated() {
        let repo = InMemoryAccountRepository::new();
        let account = Account::new(
            AccountId::new(42),
            TenantId::new(1),
            "1000".to_string(),
            "Cash".to_string(),
            AccountType::Asset,
            Currency::USD,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &account).await.unwrap();
        let result = repo.find_by_id(&ctx, AccountId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().account_code, "1000");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, AccountId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
