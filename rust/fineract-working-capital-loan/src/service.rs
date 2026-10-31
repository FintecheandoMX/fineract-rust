//! Working capital loan domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{WorkingCapitalLoan, WorkingCapitalLoanError, WorkingCapitalLoanStatus};
use crate::repository::WorkingCapitalLoanRepository;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type WorkingCapitalLoanServiceResult<T> = Result<T, WorkingCapitalLoanError>;

#[async_trait]
pub trait WorkingCapitalLoanService: Send + Sync {
    async fn create_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        loan: WorkingCapitalLoan,
    ) -> WorkingCapitalLoanServiceResult<WorkingCapitalLoan>;
    async fn get_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanServiceResult<Option<WorkingCapitalLoan>>;
    async fn list_working_capital_loans(
        &self,
        context: &AuthorizedContext,
    ) -> WorkingCapitalLoanServiceResult<Vec<WorkingCapitalLoan>>;
    async fn update_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        loan: WorkingCapitalLoan,
    ) -> WorkingCapitalLoanServiceResult<WorkingCapitalLoan>;
    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: WorkingCapitalLoanStatus,
    ) -> WorkingCapitalLoanServiceResult<()>;
    async fn delete_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanServiceResult<LoanId>;
}

pub struct WorkingCapitalLoanServiceImpl<R: WorkingCapitalLoanRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: WorkingCapitalLoanRepository> WorkingCapitalLoanServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: WorkingCapitalLoanRepository> WorkingCapitalLoanService
    for WorkingCapitalLoanServiceImpl<R>
{
    async fn create_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        loan: WorkingCapitalLoan,
    ) -> WorkingCapitalLoanServiceResult<WorkingCapitalLoan> {
        self.auth_service
            .check(context, Permission::create_loan())
            .await
            .map_err(|e| WorkingCapitalLoanError::Internal {
                detail: e.to_string(),
            })?;

        if loan.tenant_id != context.tenant_id {
            return Err(WorkingCapitalLoanError::TenantIsolationViolation);
        }

        self.repository.insert(context, &loan).await?;
        Ok(loan)
    }

    async fn get_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanServiceResult<Option<WorkingCapitalLoan>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| WorkingCapitalLoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self.repository.find_by_id(context, id).await?;

        if let Some(ref l) = loan {
            if l.tenant_id != context.tenant_id {
                return Err(WorkingCapitalLoanError::TenantIsolationViolation);
            }
        }

        Ok(loan)
    }

    async fn list_working_capital_loans(
        &self,
        context: &AuthorizedContext,
    ) -> WorkingCapitalLoanServiceResult<Vec<WorkingCapitalLoan>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| WorkingCapitalLoanError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn update_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        loan: WorkingCapitalLoan,
    ) -> WorkingCapitalLoanServiceResult<WorkingCapitalLoan> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| WorkingCapitalLoanError::Internal {
                detail: e.to_string(),
            })?;

        if loan.tenant_id != context.tenant_id {
            return Err(WorkingCapitalLoanError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, loan.id)
            .await?
            .ok_or(WorkingCapitalLoanError::NotFound(loan.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(WorkingCapitalLoanError::TenantIsolationViolation);
        }

        self.repository.update(context, &loan).await?;
        Ok(loan)
    }

    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: WorkingCapitalLoanStatus,
    ) -> WorkingCapitalLoanServiceResult<()> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| WorkingCapitalLoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(WorkingCapitalLoanError::NotFound(id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(WorkingCapitalLoanError::TenantIsolationViolation);
        }

        self.repository.update_status(context, id, status).await
    }

    async fn delete_working_capital_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> WorkingCapitalLoanServiceResult<LoanId> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| WorkingCapitalLoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(WorkingCapitalLoanError::NotFound(id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(WorkingCapitalLoanError::TenantIsolationViolation);
        }

        self.repository.delete(context, id).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn WorkingCapitalLoanService) {}
    }
}
