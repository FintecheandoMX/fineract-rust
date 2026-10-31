//! Loan domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::LoanStatus as _;
use crate::domain::{Loan, LoanCollateral, LoanError, LoanSchedule, LoanStatus, LoanTransaction};
use crate::repository::LoanRepository;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::LoanTransactionId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type LoanServiceResult<T> = Result<T, LoanError>;

#[async_trait]
pub trait LoanService: Send + Sync {
    async fn create_loan(&self, context: &AuthorizedContext, loan: Loan)
        -> LoanServiceResult<Loan>;
    async fn get_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanServiceResult<Option<Loan>>;
    async fn list_loans(&self, context: &AuthorizedContext) -> LoanServiceResult<Vec<Loan>>;
    async fn list_loans_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanServiceResult<Vec<Loan>>;
    async fn update_loan(&self, context: &AuthorizedContext, loan: Loan)
        -> LoanServiceResult<Loan>;
    async fn update_loan_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanStatus,
    ) -> LoanServiceResult<()>;
    async fn delete_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanServiceResult<LoanId>;
    async fn record_transaction(
        &self,
        context: &AuthorizedContext,
        tx: LoanTransaction,
    ) -> LoanServiceResult<LoanTransactionId>;
    async fn generate_schedule(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
        schedules: Vec<LoanSchedule>,
    ) -> LoanServiceResult<()>;
}

pub struct LoanServiceImpl<R: LoanRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: LoanRepository> LoanServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: LoanRepository> LoanService for LoanServiceImpl<R> {
    async fn create_loan(
        &self,
        context: &AuthorizedContext,
        loan: Loan,
    ) -> LoanServiceResult<Loan> {
        self.auth_service
            .check(context, Permission::create_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        if loan.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        self.repository.insert(context, &loan).await?;
        Ok(loan)
    }

    async fn get_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanServiceResult<Option<Loan>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self.repository.find_by_id(context, id).await?;

        if let Some(ref l) = loan {
            if l.tenant_id != context.tenant_id {
                return Err(LoanError::TenantIsolationViolation);
            }
        }

        Ok(loan)
    }

    async fn list_loans(&self, context: &AuthorizedContext) -> LoanServiceResult<Vec<Loan>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn list_loans_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanServiceResult<Vec<Loan>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        let loans = self
            .repository
            .find_all_by_client(context, client_id)
            .await?;

        Ok(loans
            .into_iter()
            .filter(|l| l.tenant_id == context.tenant_id)
            .collect())
    }

    async fn update_loan(
        &self,
        context: &AuthorizedContext,
        loan: Loan,
    ) -> LoanServiceResult<Loan> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        if loan.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, loan.id)
            .await?
            .ok_or(LoanError::NotFound(loan.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        if existing.status == LoanStatus::Closed || existing.status == LoanStatus::WrittenOff {
            return Err(LoanError::CannotModify);
        }

        self.repository.update(context, &loan).await?;
        Ok(loan)
    }

    async fn update_loan_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanStatus,
    ) -> LoanServiceResult<()> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(LoanError::NotFound(id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        self.repository.update_status(context, id, status).await
    }

    async fn delete_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanServiceResult<LoanId> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(LoanError::NotFound(id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        self.repository.delete(context, id).await?;
        Ok(id)
    }

    async fn record_transaction(
        &self,
        context: &AuthorizedContext,
        tx: LoanTransaction,
    ) -> LoanServiceResult<LoanTransactionId> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        if tx.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        // Verify loan belongs to tenant
        let loan = self
            .repository
            .find_by_id(context, tx.loan_id)
            .await?
            .ok_or(LoanError::NotFound(tx.loan_id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        Ok(tx.id)
    }

    async fn generate_schedule(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
        schedules: Vec<LoanSchedule>,
    ) -> LoanServiceResult<()> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self
            .repository
            .find_by_id(context, loan_id)
            .await?
            .ok_or(LoanError::NotFound(loan_id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(LoanError::TenantIsolationViolation);
        }

        for schedule in &schedules {
            if schedule.tenant_id != context.tenant_id {
                return Err(LoanError::TenantIsolationViolation);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn LoanService) {}
    }
}
