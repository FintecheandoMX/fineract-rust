//! Loan repositories with tenant isolation.

use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::Row;

use crate::domain::{
    Loan, LoanCollateral, LoanError, LoanSchedule, LoanStatus, LoanTermPeriod, LoanTransaction,
    LoanType,
};
use fineract_core::ids::ClientId;
use fineract_core::ids::CollateralId;
use fineract_core::ids::CollateralId as _;
use fineract_core::ids::LoanId;
use fineract_core::ids::LoanScheduleId;
use fineract_core::ids::LoanTransactionId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type LoanRepositoryResult<T> = Result<T, LoanError>;

#[async_trait]
pub trait LoanRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanRepositoryResult<Option<Loan>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> LoanRepositoryResult<Vec<Loan>>;
    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanRepositoryResult<Vec<Loan>>;
    async fn insert(&self, context: &AuthorizedContext, loan: &Loan) -> LoanRepositoryResult<()>;
    async fn update(&self, context: &AuthorizedContext, loan: &Loan) -> LoanRepositoryResult<()>;
    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanStatus,
    ) -> LoanRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: LoanId) -> LoanRepositoryResult<()>;
}

#[async_trait]
pub trait LoanTransactionRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanTransactionId,
    ) -> LoanRepositoryResult<Option<LoanTransaction>>;
    async fn find_all_by_loan(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> LoanRepositoryResult<Vec<LoanTransaction>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        tx: &LoanTransaction,
    ) -> LoanRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        tx: &LoanTransaction,
    ) -> LoanRepositoryResult<()>;
}

#[async_trait]
pub trait LoanScheduleRepository: Send + Sync {
    async fn find_all_by_loan(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> LoanRepositoryResult<Vec<LoanSchedule>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        schedule: &LoanSchedule,
    ) -> LoanRepositoryResult<()>;
    async fn insert_batch(
        &self,
        context: &AuthorizedContext,
        schedules: &[LoanSchedule],
    ) -> LoanRepositoryResult<()>;
}

pub struct SqlxLoanRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxLoanRepository {
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
    ) -> LoanRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| LoanError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl LoanRepository for SqlxLoanRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanRepositoryResult<Option<Loan>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, office_id, client_id, loan_officer_id, loan_type, product_id, principal, currency, interest_rate, term_count, term_period, repayment_count, repayment_every, disbursement_date, status, external_id, active FROM loans WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| LoanError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_loan(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> LoanRepositoryResult<Vec<Loan>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, client_id, loan_officer_id, loan_type, product_id, principal, currency, interest_rate, term_count, term_period, repayment_count, repayment_every, disbursement_date, status, external_id, active FROM loans WHERE tenant_id = $1 ORDER BY disbursement_date DESC",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| LoanError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_loan(&r)).collect()
    }

    async fn find_all_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanRepositoryResult<Vec<Loan>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, client_id, loan_officer_id, loan_type, product_id, principal, currency, interest_rate, term_count, term_period, repayment_count, repayment_every, disbursement_date, status, external_id, active FROM loans WHERE tenant_id = $1 AND client_id = $2 ORDER BY disbursement_date DESC",
        )
        .bind(context.tenant_id.value())
        .bind(client_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| LoanError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_loan(&r)).collect()
    }

    async fn insert(&self, context: &AuthorizedContext, loan: &Loan) -> LoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO loans (id, tenant_id, office_id, client_id, loan_officer_id, loan_type, product_id, principal, currency, interest_rate, term_count, term_period, repayment_count, repayment_every, disbursement_date, status, external_id, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
        )
        .bind(loan.id.value())
        .bind(loan.tenant_id.value())
        .bind(loan.office_id.value())
        .bind(loan.client_id.value())
        .bind(loan.loan_officer_id.map(|id| id.value()))
        .bind(format!("{:?}", loan.loan_type))
        .bind(loan.product_id)
        .bind(loan.principal.to_string())
        .bind(loan.currency.code())
        .bind(loan.interest_rate.to_string())
        .bind(loan.term_count)
        .bind(format!("{:?}", loan.term_period))
        .bind(loan.repayment_count)
        .bind(loan.repayment_every)
        .bind(loan.disbursement_date)
        .bind(format!("{:?}", loan.status))
        .bind(&loan.external_id)
        .bind(loan.active)
        .execute(&pool)
        .await
        .map_err(|e| LoanError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(&self, context: &AuthorizedContext, loan: &Loan) -> LoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE loans SET office_id = $2, client_id = $3, loan_officer_id = $4, loan_type = $5, product_id = $6, principal = $7, currency = $8, interest_rate = $9, term_count = $10, term_period = $11, repayment_count = $12, repayment_every = $13, disbursement_date = $14, status = $15, external_id = $16, active = $17 WHERE id = $1 AND tenant_id = $18",
        )
        .bind(loan.id.value())
        .bind(loan.office_id.value())
        .bind(loan.client_id.value())
        .bind(loan.loan_officer_id.map(|id| id.value()))
        .bind(format!("{:?}", loan.loan_type))
        .bind(loan.product_id)
        .bind(loan.principal.to_string())
        .bind(loan.currency.code())
        .bind(loan.interest_rate.to_string())
        .bind(loan.term_count)
        .bind(format!("{:?}", loan.term_period))
        .bind(loan.repayment_count)
        .bind(loan.repayment_every)
        .bind(loan.disbursement_date)
        .bind(format!("{:?}", loan.status))
        .bind(&loan.external_id)
        .bind(loan.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| LoanError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(LoanError::NotFound(loan.id));
        }
        Ok(())
    }

    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanStatus,
    ) -> LoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("UPDATE loans SET status = $2 WHERE id = $1 AND tenant_id = $3")
            .bind(id.value())
            .bind(format!("{:?}", status))
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| LoanError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(LoanError::NotFound(id));
        }
        Ok(())
    }

    async fn delete(&self, context: &AuthorizedContext, id: LoanId) -> LoanRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM loans WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| LoanError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(LoanError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_loan(row: &sqlx::postgres::PgRow) -> LoanRepositoryResult<Loan> {
    Ok(Loan {
        id: LoanId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| LoanError::QueryError {
            detail: "invalid loan id".to_string(),
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            LoanError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        office_id: OfficeId::try_from(row.try_get::<i64, _>("office_id")?).map_err(|_| {
            LoanError::QueryError {
                detail: "invalid office id".to_string(),
            }
        })?,
        client_id: ClientId::try_from(row.try_get::<i64, _>("client_id")?).map_err(|_| {
            LoanError::QueryError {
                detail: "invalid client id".to_string(),
            }
        })?,
        loan_officer_id: row
            .try_get::<Option<i64>, _>("loan_officer_id")?
            .map(StaffId::try_from)
            .transpose()
            .map_err(|_| LoanError::QueryError {
                detail: "invalid staff id".to_string(),
            })?,
        loan_type: match row.try_get::<String, _>("loan_type")?.as_str() {
            "Individual" => LoanType::Individual,
            "Group" => LoanType::Group,
            _ => {
                return Err(LoanError::QueryError {
                    detail: "invalid loan type".to_string(),
                })
            }
        },
        product_id: row.try_get("product_id")?,
        principal: rust_decimal::Decimal::from_str_exact(&row.try_get::<String, _>("principal")?)
            .map_err(|_| LoanError::QueryError {
            detail: "invalid principal".to_string(),
        })?,
        currency: fineract_core::money::Currency::from_code(&row.try_get::<String, _>("currency")?)
            .map_err(|_| LoanError::QueryError {
                detail: "invalid currency".to_string(),
            })?,
        interest_rate: rust_decimal::Decimal::from_str_exact(
            &row.try_get::<String, _>("interest_rate")?,
        )
        .map_err(|_| LoanError::QueryError {
            detail: "invalid interest rate".to_string(),
        })?,
        term_count: row.try_get("term_count")?,
        term_period: match row.try_get::<String, _>("term_period")?.as_str() {
            "Days" => LoanTermPeriod::Days,
            "Weeks" => LoanTermPeriod::Weeks,
            "Months" => LoanTermPeriod::Months,
            "Years" => LoanTermPeriod::Years,
            _ => {
                return Err(LoanError::QueryError {
                    detail: "invalid term period".to_string(),
                })
            }
        },
        repayment_count: row.try_get("repayment_count")?,
        repayment_every: row.try_get("repayment_every")?,
        disbursement_date: row.try_get("disbursement_date")?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "PendingApproval" => LoanStatus::PendingApproval,
            "Approved" => LoanStatus::Approved,
            "Active" => LoanStatus::Active,
            "Overpaid" => LoanStatus::Overpaid,
            "Closed" => LoanStatus::Closed,
            "Rejected" => LoanStatus::Rejected,
            "WrittenOff" => LoanStatus::WrittenOff,
            _ => {
                return Err(LoanError::QueryError {
                    detail: "invalid loan status".to_string(),
                })
            }
        },
        external_id: row.try_get("external_id")?,
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

    pub struct InMemoryLoanRepository {
        store: Mutex<HashMap<(TenantId, LoanId), Loan>>,
    }

    impl Default for InMemoryLoanRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryLoanRepository {
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
    impl LoanRepository for InMemoryLoanRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> LoanRepositoryResult<Option<Loan>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> LoanRepositoryResult<Vec<Loan>> {
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
        ) -> LoanRepositoryResult<Vec<Loan>> {
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
            loan: &Loan,
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, loan.id), loan.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            loan: &Loan,
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, loan.id)) {
                store.insert(Self::key(context.tenant_id, loan.id), loan.clone());
                Ok(())
            } else {
                Err(LoanError::NotFound(loan.id))
            }
        }

        async fn update_status(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
            status: LoanStatus,
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(loan) = store.get_mut(&Self::key(context.tenant_id, id)) {
                loan.status = status;
                Ok(())
            } else {
                Err(LoanError::NotFound(id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: LoanId,
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(LoanError::NotFound(id))
            }
        }
    }

    pub struct InMemoryLoanTransactionRepository {
        store: Mutex<HashMap<(TenantId, LoanTransactionId), LoanTransaction>>,
    }

    impl Default for InMemoryLoanTransactionRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryLoanTransactionRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: LoanTransactionId) -> (TenantId, LoanTransactionId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl LoanTransactionRepository for InMemoryLoanTransactionRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: LoanTransactionId,
        ) -> LoanRepositoryResult<Option<LoanTransaction>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_loan(
            &self,
            context: &AuthorizedContext,
            _loan_id: LoanId,
        ) -> LoanRepositoryResult<Vec<LoanTransaction>> {
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
            tx: &LoanTransaction,
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, tx.id), tx.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            tx: &LoanTransaction,
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, tx.id)) {
                store.insert(Self::key(context.tenant_id, tx.id), tx.clone());
                Ok(())
            } else {
                Err(LoanError::NotFound(LoanId::new(0)))
            }
        }
    }

    pub struct InMemoryLoanScheduleRepository {
        store: Mutex<HashMap<(TenantId, LoanScheduleId), LoanSchedule>>,
    }

    impl Default for InMemoryLoanScheduleRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryLoanScheduleRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: LoanScheduleId) -> (TenantId, LoanScheduleId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl LoanScheduleRepository for InMemoryLoanScheduleRepository {
        async fn find_all_by_loan(
            &self,
            context: &AuthorizedContext,
            _loan_id: LoanId,
        ) -> LoanRepositoryResult<Vec<LoanSchedule>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), s)| {
                    if *tid == context.tenant_id {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            schedule: &LoanSchedule,
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, schedule.id), schedule.clone());
            Ok(())
        }

        async fn insert_batch(
            &self,
            context: &AuthorizedContext,
            schedules: &[LoanSchedule],
        ) -> LoanRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            for schedule in schedules {
                store.insert(Self::key(context.tenant_id, schedule.id), schedule.clone());
            }
            Ok(())
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
    async fn loan_repository_is_tenant_isolated() {
        let repo = InMemoryLoanRepository::new();
        let loan = Loan::new(
            LoanId::new(42),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            LoanType::Individual,
            rust_decimal::Decimal::new(100000, 2),
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
            12,
            LoanTermPeriod::Months,
            chrono::Utc::now().date_naive(),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &loan).await.unwrap();
        let result = repo.find_by_id(&ctx, LoanId::new(42)).await.unwrap();
        assert_eq!(
            result.unwrap().principal,
            rust_decimal::Decimal::new(100000, 2)
        );

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, LoanId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn loan_schedule_repository_is_tenant_isolated() {
        let repo = InMemoryLoanScheduleRepository::new();
        let schedule = LoanSchedule {
            id: LoanScheduleId::new(42),
            loan_id: LoanId::new(1),
            tenant_id: TenantId::new(1),
            installment_number: 1,
            due_date: chrono::Utc::now().date_naive(),
            principal_due: rust_decimal::Decimal::new(833333, 2),
            interest_due: rust_decimal::Decimal::new(83333, 2),
            total_due: rust_decimal::Decimal::new(916666, 2),
            principal_paid: rust_decimal::Decimal::ZERO,
            interest_paid: rust_decimal::Decimal::ZERO,
            principal_outstanding: rust_decimal::Decimal::new(1000000, 2),
            interest_outstanding: rust_decimal::Decimal::new(83333, 2),
            penalty: rust_decimal::Decimal::ZERO,
            is_paid: false,
        };

        let ctx = make_context(1);
        repo.insert(&ctx, &schedule).await.unwrap();
        let result = repo.find_all_by_loan(&ctx, LoanId::new(1)).await.unwrap();
        assert_eq!(result.len(), 1);

        let ctx2 = make_context(2);
        let result = repo.find_all_by_loan(&ctx2, LoanId::new(1)).await.unwrap();
        assert_eq!(result.len(), 0);
    }
}
