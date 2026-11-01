//! Savings product repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{SavingsProduct, SavingsProductError};
use fineract_core::ids::SavingsProductId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type SavingsProductRepositoryResult<T> = Result<T, SavingsProductError>;

#[async_trait]
pub trait SavingsProductRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductRepositoryResult<Option<SavingsProduct>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsProductRepositoryResult<Vec<SavingsProduct>>;
    async fn find_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> SavingsProductRepositoryResult<Option<SavingsProduct>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        product: &SavingsProduct,
    ) -> SavingsProductRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        product: &SavingsProduct,
    ) -> SavingsProductRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductRepositoryResult<()>;
}

pub struct SqlxSavingsProductRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxSavingsProductRepository {
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
    ) -> SavingsProductRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| SavingsProductError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl SavingsProductRepository for SqlxSavingsProductRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductRepositoryResult<Option<SavingsProduct>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, short_name, description, currency_code, currency_digits, interest_rate, interest_compounding_period_type, interest_posting_period_type, interest_calculation_type, interest_calculation_days_in_year_type, min_required_opening_balance, max_balance, min_deposit_amount, max_deposit_amount, min_withdrawal_amount, max_withdrawal_amount, allow_overdraft, overdraft_limit, nominal_annual_interest_rate, annual_interest_rate, min_balance_to_earn_interest, account_fee, maintenance_fee, withdrawal_fee, deposit_fee, inactive_account_fee, reject_if_insufficient_funds, is_active, created_at, updated_at FROM savings_products WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| SavingsProductError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_savings_product(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsProductRepositoryResult<Vec<SavingsProduct>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, short_name, description, currency_code, currency_digits, interest_rate, interest_compounding_period_type, interest_posting_period_type, interest_calculation_type, interest_calculation_days_in_year_type, min_required_opening_balance, max_balance, min_deposit_amount, max_deposit_amount, min_withdrawal_amount, max_withdrawal_amount, allow_overdraft, overdraft_limit, nominal_annual_interest_rate, annual_interest_rate, min_balance_to_earn_interest, account_fee, maintenance_fee, withdrawal_fee, deposit_fee, inactive_account_fee, reject_if_insufficient_funds, is_active, created_at, updated_at FROM savings_products WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| SavingsProductError::QueryError { detail: e.to_string() })?;

        rows.into_iter()
            .map(|r| map_row_to_savings_product(&r))
            .collect()
    }

    async fn find_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> SavingsProductRepositoryResult<Option<SavingsProduct>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, short_name, description, currency_code, currency_digits, interest_rate, interest_compounding_period_type, interest_posting_period_type, interest_calculation_type, interest_calculation_days_in_year_type, min_required_opening_balance, max_balance, min_deposit_amount, max_deposit_amount, min_withdrawal_amount, max_withdrawal_amount, allow_overdraft, overdraft_limit, nominal_annual_interest_rate, annual_interest_rate, min_balance_to_earn_interest, account_fee, maintenance_fee, withdrawal_fee, deposit_fee, inactive_account_fee, reject_if_insufficient_funds, is_active, created_at, updated_at FROM savings_products WHERE name = $1 AND tenant_id = $2",
        )
        .bind(name)
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| SavingsProductError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_savings_product(&r)).transpose()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        product: &SavingsProduct,
    ) -> SavingsProductRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO savings_products (id, tenant_id, name, short_name, description, currency_code, currency_digits, interest_rate, interest_compounding_period_type, interest_posting_period_type, interest_calculation_type, interest_calculation_days_in_year_type, min_required_opening_balance, max_balance, min_deposit_amount, max_deposit_amount, min_withdrawal_amount, max_withdrawal_amount, allow_overdraft, overdraft_limit, nominal_annual_interest_rate, annual_interest_rate, min_balance_to_earn_interest, account_fee, maintenance_fee, withdrawal_fee, deposit_fee, inactive_account_fee, reject_if_insufficient_funds, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31)",
        )
        .bind(product.id.value())
        .bind(product.tenant_id.value())
        .bind(&product.name)
        .bind(&product.short_name)
        .bind(&product.description)
        .bind(&product.currency_code)
        .bind(product.currency_digits)
        .bind(&product.interest_rate)
        .bind(product.interest_compounding_period_type)
        .bind(product.interest_posting_period_type)
        .bind(product.interest_calculation_type)
        .bind(product.interest_calculation_days_in_year_type)
        .bind(&product.min_required_opening_balance)
        .bind(&product.max_balance)
        .bind(&product.min_deposit_amount)
        .bind(&product.max_deposit_amount)
        .bind(&product.min_withdrawal_amount)
        .bind(&product.max_withdrawal_amount)
        .bind(product.allow_overdraft)
        .bind(&product.overdraft_limit)
        .bind(&product.nominal_annual_interest_rate)
        .bind(&product.annual_interest_rate)
        .bind(&product.min_balance_to_earn_interest)
        .bind(&product.account_fee)
        .bind(&product.maintenance_fee)
        .bind(&product.withdrawal_fee)
        .bind(&product.deposit_fee)
        .bind(&product.inactive_account_fee)
        .bind(product.reject_if_insufficient_funds)
        .bind(product.is_active)
        .bind(product.created_at)
        .bind(product.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| SavingsProductError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        product: &SavingsProduct,
    ) -> SavingsProductRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE savings_products SET name = $2, short_name = $3, description = $4, currency_code = $5, currency_digits = $6, interest_rate = $7, interest_compounding_period_type = $8, interest_posting_period_type = $9, interest_calculation_type = $10, interest_calculation_days_in_year_type = $11, min_required_opening_balance = $12, max_balance = $13, min_deposit_amount = $14, max_deposit_amount = $15, min_withdrawal_amount = $16, max_withdrawal_amount = $17, allow_overdraft = $18, overdraft_limit = $19, nominal_annual_interest_rate = $20, annual_interest_rate = $21, min_balance_to_earn_interest = $22, account_fee = $23, maintenance_fee = $24, withdrawal_fee = $25, deposit_fee = $26, inactive_account_fee = $27, reject_if_insufficient_funds = $28, is_active = $29, updated_at = $30 WHERE id = $1 AND tenant_id = $31",
        )
        .bind(product.id.value())
        .bind(&product.name)
        .bind(&product.short_name)
        .bind(&product.description)
        .bind(&product.currency_code)
        .bind(product.currency_digits)
        .bind(&product.interest_rate)
        .bind(product.interest_compounding_period_type)
        .bind(product.interest_posting_period_type)
        .bind(product.interest_calculation_type)
        .bind(product.interest_calculation_days_in_year_type)
        .bind(&product.min_required_opening_balance)
        .bind(&product.max_balance)
        .bind(&product.min_deposit_amount)
        .bind(&product.max_deposit_amount)
        .bind(&product.min_withdrawal_amount)
        .bind(&product.max_withdrawal_amount)
        .bind(product.allow_overdraft)
        .bind(&product.overdraft_limit)
        .bind(&product.nominal_annual_interest_rate)
        .bind(&product.annual_interest_rate)
        .bind(&product.min_balance_to_earn_interest)
        .bind(&product.account_fee)
        .bind(&product.maintenance_fee)
        .bind(&product.withdrawal_fee)
        .bind(&product.deposit_fee)
        .bind(&product.inactive_account_fee)
        .bind(product.reject_if_insufficient_funds)
        .bind(product.is_active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| SavingsProductError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(SavingsProductError::NotFound(product.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM savings_products WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| SavingsProductError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(SavingsProductError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_savings_product(
    row: &sqlx::postgres::PgRow,
) -> SavingsProductRepositoryResult<SavingsProduct> {
    Ok(SavingsProduct {
        id: SavingsProductId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            SavingsProductError::QueryError {
                detail: "invalid savings product id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            SavingsProductError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        short_name: row.try_get("short_name")?,
        description: row.try_get("description")?,
        currency_code: row.try_get("currency_code")?,
        currency_digits: row.try_get("currency_digits")?,
        interest_rate: row.try_get("interest_rate")?,
        interest_compounding_period_type: row.try_get("interest_compounding_period_type")?,
        interest_posting_period_type: row.try_get("interest_posting_period_type")?,
        interest_calculation_type: row.try_get("interest_calculation_type")?,
        interest_calculation_days_in_year_type: row
            .try_get("interest_calculation_days_in_year_type")?,
        min_required_opening_balance: row.try_get("min_required_opening_balance")?,
        max_balance: row.try_get("max_balance")?,
        min_deposit_amount: row.try_get("min_deposit_amount")?,
        max_deposit_amount: row.try_get("max_deposit_amount")?,
        min_withdrawal_amount: row.try_get("min_withdrawal_amount")?,
        max_withdrawal_amount: row.try_get("max_withdrawal_amount")?,
        allow_overdraft: row.try_get("allow_overdraft")?,
        overdraft_limit: row.try_get("overdraft_limit")?,
        nominal_annual_interest_rate: row.try_get("nominal_annual_interest_rate")?,
        annual_interest_rate: row.try_get("annual_interest_rate")?,
        min_balance_to_earn_interest: row.try_get("min_balance_to_earn_interest")?,
        account_fee: row.try_get("account_fee")?,
        maintenance_fee: row.try_get("maintenance_fee")?,
        withdrawal_fee: row.try_get("withdrawal_fee")?,
        deposit_fee: row.try_get("deposit_fee")?,
        inactive_account_fee: row.try_get("inactive_account_fee")?,
        reject_if_insufficient_funds: row.try_get("reject_if_insufficient_funds")?,
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

    pub struct InMemorySavingsProductRepository {
        store: Mutex<HashMap<(TenantId, SavingsProductId), SavingsProduct>>,
    }

    impl Default for InMemorySavingsProductRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemorySavingsProductRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: SavingsProductId) -> (TenantId, SavingsProductId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl SavingsProductRepository for InMemorySavingsProductRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: SavingsProductId,
        ) -> SavingsProductRepositoryResult<Option<SavingsProduct>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> SavingsProductRepositoryResult<Vec<SavingsProduct>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), p)| {
                    if *tid == context.tenant_id {
                        Some(p.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_by_name(
            &self,
            context: &AuthorizedContext,
            name: &str,
        ) -> SavingsProductRepositoryResult<Option<SavingsProduct>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), p)| {
                    if *tid == context.tenant_id && p.name == name {
                        Some(p.clone())
                    } else {
                        None
                    }
                })
                .next())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            product: &SavingsProduct,
        ) -> SavingsProductRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, product.id), product.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            product: &SavingsProduct,
        ) -> SavingsProductRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, product.id)) {
                store.insert(Self::key(context.tenant_id, product.id), product.clone());
                Ok(())
            } else {
                Err(SavingsProductError::NotFound(product.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: SavingsProductId,
        ) -> SavingsProductRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(SavingsProductError::NotFound(id))
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
    async fn savings_product_repository_is_tenant_isolated() {
        let repo = InMemorySavingsProductRepository::new();
        let product = SavingsProduct::new(
            SavingsProductId::new(42),
            TenantId::new(1),
            "Test Savings".to_string(),
            "TS".to_string(),
            None,
            "USD".to_string(),
            "5".to_string(),
            "5".to_string(),
            "5".to_string(),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &product).await.unwrap();
        let result = repo
            .find_by_id(&ctx, SavingsProductId::new(42))
            .await
            .unwrap();
        assert_eq!(result.unwrap().name, "Test Savings");

        let ctx2 = make_context(2);
        let result = repo
            .find_by_id(&ctx2, SavingsProductId::new(42))
            .await
            .unwrap();
        assert_eq!(result, None);
    }
}
