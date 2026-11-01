//! Loan product repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{LoanProduct, LoanProductError};
use fineract_core::ids::LoanProductId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type LoanProductRepositoryResult<T> = Result<T, LoanProductError>;

#[async_trait]
pub trait LoanProductRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductRepositoryResult<Option<LoanProduct>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> LoanProductRepositoryResult<Vec<LoanProduct>>;
    async fn find_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> LoanProductRepositoryResult<Option<LoanProduct>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        product: &LoanProduct,
    ) -> LoanProductRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        product: &LoanProduct,
    ) -> LoanProductRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductRepositoryResult<()>;
}

pub struct SqlxLoanProductRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxLoanProductRepository {
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
    ) -> LoanProductRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| LoanProductError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl LoanProductRepository for SqlxLoanProductRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductRepositoryResult<Option<LoanProduct>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, short_name, description, principal, number_of_repayments, repayment_every, repayment_frequency_type, amortization_type, interest_rate_per_period, interest_type, interest_calculation_period_type, is_equal_amortization, min_principal, max_principal, min_number_of_repayments, max_number_of_repayments, min_interest_rate_per_period, max_interest_rate_per_period, currency_code, currency_digits, in_arrears_tolerance, days_in_year_type, days_in_month_type, include_in_borrower_cycle, use_borrower_cycle, number_of_borrower_cycle, is_active, created_at, updated_at FROM loan_products WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| LoanProductError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_loan_product(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> LoanProductRepositoryResult<Vec<LoanProduct>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, short_name, description, principal, number_of_repayments, repayment_every, repayment_frequency_type, amortization_type, interest_rate_per_period, interest_type, interest_calculation_period_type, is_equal_amortization, min_principal, max_principal, min_number_of_repayments, max_number_of_repayments, min_interest_rate_per_period, max_interest_rate_per_period, currency_code, currency_digits, in_arrears_tolerance, days_in_year_type, days_in_month_type, include_in_borrower_cycle, use_borrower_cycle, number_of_borrower_cycle, is_active, created_at, updated_at FROM loan_products WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| LoanProductError::QueryError { detail: e.to_string() })?;

        rows.into_iter()
            .map(|r| map_row_to_loan_product(&r))
            .collect()
    }

    async fn find_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> LoanProductRepositoryResult<Option<LoanProduct>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, short_name, description, principal, number_of_repayments, repayment_every, repayment_frequency_type, amortization_type, interest_rate_per_period, interest_type, interest_calculation_period_type, is_equal_amortization, min_principal, max_principal, min_number_of_repayments, max_number_of_repayments, min_interest_rate_per_period, max_interest_rate_per_period, currency_code, currency_digits, in_arrears_tolerance, days_in_year_type, days_in_month_type, include_in_borrower_cycle, use_borrower_cycle, number_of_borrower_cycle, is_active, created_at, updated_at FROM loan_products WHERE name = $1 AND tenant_id = $2",
        )
        .bind(name)
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| LoanProductError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_loan_product(&r)).transpose()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        product: &LoanProduct,
    ) -> LoanProductRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO loan_products (id, tenant_id, name, short_name, description, principal, number_of_repayments, repayment_every, repayment_frequency_type, amortization_type, interest_rate_per_period, interest_type, interest_calculation_period_type, is_equal_amortization, min_principal, max_principal, min_number_of_repayments, max_number_of_repayments, min_interest_rate_per_period, max_interest_rate_per_period, currency_code, currency_digits, in_arrears_tolerance, days_in_year_type, days_in_month_type, include_in_borrower_cycle, use_borrower_cycle, number_of_borrower_cycle, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31)",
        )
        .bind(product.id.value())
        .bind(product.tenant_id.value())
        .bind(&product.name)
        .bind(&product.short_name)
        .bind(&product.description)
        .bind(&product.principal)
        .bind(product.number_of_repayments)
        .bind(product.repayment_every)
        .bind(product.repayment_frequency_type)
        .bind(product.amortization_type)
        .bind(&product.interest_rate_per_period)
        .bind(product.interest_type)
        .bind(product.interest_calculation_period_type)
        .bind(product.is_equal_amortization)
        .bind(&product.min_principal)
        .bind(&product.max_principal)
        .bind(product.min_number_of_repayments)
        .bind(product.max_number_of_repayments)
        .bind(&product.min_interest_rate_per_period)
        .bind(&product.max_interest_rate_per_period)
        .bind(&product.currency_code)
        .bind(product.currency_digits)
        .bind(product.in_arrears_tolerance)
        .bind(product.days_in_year_type)
        .bind(product.days_in_month_type)
        .bind(product.include_in_borrower_cycle)
        .bind(product.use_borrower_cycle)
        .bind(product.number_of_borrower_cycle)
        .bind(product.is_active)
        .bind(product.created_at)
        .bind(product.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| LoanProductError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        product: &LoanProduct,
    ) -> LoanProductRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE loan_products SET name = $2, short_name = $3, description = $4, principal = $5, number_of_repayments = $6, repayment_every = $7, repayment_frequency_type = $8, amortization_type = $9, interest_rate_per_period = $10, interest_type = $11, interest_calculation_period_type = $12, is_equal_amortization = $13, min_principal = $14, max_principal = $15, min_number_of_repayments = $16, max_number_of_repayments = $17, min_interest_rate_per_period = $18, max_interest_rate_per_period = $19, currency_code = $20, currency_digits = $21, in_arrears_tolerance = $22, days_in_year_type = $23, days_in_month_type = $24, include_in_borrower_cycle = $25, use_borrower_cycle = $26, number_of_borrower_cycle = $27, is_active = $28, updated_at = $29 WHERE id = $1 AND tenant_id = $30",
        )
        .bind(product.id.value())
        .bind(&product.name)
        .bind(&product.short_name)
        .bind(&product.description)
        .bind(&product.principal)
        .bind(product.number_of_repayments)
        .bind(product.repayment_every)
        .bind(product.repayment_frequency_type)
        .bind(product.amortization_type)
        .bind(&product.interest_rate_per_period)
        .bind(product.interest_type)
        .bind(product.interest_calculation_period_type)
        .bind(product.is_equal_amortization)
        .bind(&product.min_principal)
        .bind(&product.max_principal)
        .bind(product.min_number_of_repayments)
        .bind(product.max_number_of_repayments)
        .bind(&product.min_interest_rate_per_period)
        .bind(&product.max_interest_rate_per_period)
        .bind(&product.currency_code)
        .bind(product.currency_digits)
        .bind(product.in_arrears_tolerance)
        .bind(product.days_in_year_type)
        .bind(product.days_in_month_type)
        .bind(product.include_in_borrower_cycle)
        .bind(product.use_borrower_cycle)
        .bind(product.number_of_borrower_cycle)
        .bind(product.is_active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| LoanProductError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(LoanProductError::NotFound(product.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM loan_products WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| LoanProductError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(LoanProductError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_loan_product(
    row: &sqlx::postgres::PgRow,
) -> LoanProductRepositoryResult<LoanProduct> {
    Ok(LoanProduct {
        id: LoanProductId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            LoanProductError::QueryError {
                detail: "invalid loan product id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            LoanProductError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        short_name: row.try_get("short_name")?,
        description: row.try_get("description")?,
        principal: row.try_get("principal")?,
        number_of_repayments: row.try_get("number_of_repayments")?,
        repayment_every: row.try_get("repayment_every")?,
        repayment_frequency_type: row.try_get("repayment_frequency_type")?,
        amortization_type: row.try_get("amortization_type")?,
        interest_rate_per_period: row.try_get("interest_rate_per_period")?,
        interest_type: row.try_get("interest_type")?,
        interest_calculation_period_type: row.try_get("interest_calculation_period_type")?,
        is_equal_amortization: row.try_get("is_equal_amortization")?,
        min_principal: row.try_get("min_principal")?,
        max_principal: row.try_get("max_principal")?,
        min_number_of_repayments: row.try_get("min_number_of_repayments")?,
        max_number_of_repayments: row.try_get("max_number_of_repayments")?,
        min_interest_rate_per_period: row.try_get("min_interest_rate_per_period")?,
        max_interest_rate_per_period: row.try_get("max_interest_rate_per_period")?,
        currency_code: row.try_get("currency_code")?,
        currency_digits: row.try_get("currency_digits")?,
        in_arrears_tolerance: row.try_get("in_arrears_tolerance")?,
        days_in_year_type: row.try_get("days_in_year_type")?,
        days_in_month_type: row.try_get("days_in_month_type")?,
        include_in_borrower_cycle: row.try_get("include_in_borrower_cycle")?,
        use_borrower_cycle: row.try_get("use_borrower_cycle")?,
        number_of_borrower_cycle: row.try_get("number_of_borrower_cycle")?,
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

    pub struct InMemoryLoanProductRepository {
        store: Mutex<HashMap<(TenantId, LoanProductId), LoanProduct>>,
    }

    impl Default for InMemoryLoanProductRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryLoanProductRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: LoanProductId) -> (TenantId, LoanProductId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl LoanProductRepository for InMemoryLoanProductRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: LoanProductId,
        ) -> LoanProductRepositoryResult<Option<LoanProduct>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> LoanProductRepositoryResult<Vec<LoanProduct>> {
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
        ) -> LoanProductRepositoryResult<Option<LoanProduct>> {
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
            product: &LoanProduct,
        ) -> LoanProductRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, product.id), product.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            product: &LoanProduct,
        ) -> LoanProductRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, product.id)) {
                store.insert(Self::key(context.tenant_id, product.id), product.clone());
                Ok(())
            } else {
                Err(LoanProductError::NotFound(product.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: LoanProductId,
        ) -> LoanProductRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(LoanProductError::NotFound(id))
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
    async fn loan_product_repository_is_tenant_isolated() {
        let repo = InMemoryLoanProductRepository::new();
        let product = LoanProduct::new(
            LoanProductId::new(42),
            TenantId::new(1),
            "Test Loan".to_string(),
            "TL".to_string(),
            None,
            "1000".to_string(),
            12,
            1,
            2,
            1,
            "10".to_string(),
            1,
            1,
            "USD".to_string(),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &product).await.unwrap();
        let result = repo.find_by_id(&ctx, LoanProductId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Test Loan");

        let ctx2 = make_context(2);
        let result = repo
            .find_by_id(&ctx2, LoanProductId::new(42))
            .await
            .unwrap();
        assert_eq!(result, None);
    }
}
