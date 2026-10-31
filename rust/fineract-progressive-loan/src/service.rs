//! Progressive loan domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{ProgressiveLoan, ProgressiveLoanError, ProgressiveLoanPhase};
use crate::repository::ProgressiveLoanRepository;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type ProgressiveLoanServiceResult<T> = Result<T, ProgressiveLoanError>;

#[async_trait]
pub trait ProgressiveLoanService: Send + Sync {
    async fn create_progressive_loan(
        &self,
        context: &AuthorizedContext,
        loan: ProgressiveLoan,
    ) -> ProgressiveLoanServiceResult<ProgressiveLoan>;
    async fn get_progressive_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanServiceResult<Option<ProgressiveLoan>>;
    async fn list_progressive_loans(
        &self,
        context: &AuthorizedContext,
    ) -> ProgressiveLoanServiceResult<Vec<ProgressiveLoan>>;
    async fn update_progressive_loan(
        &self,
        context: &AuthorizedContext,
        loan: ProgressiveLoan,
    ) -> ProgressiveLoanServiceResult<ProgressiveLoan>;
    async fn advance_phase(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        phase: ProgressiveLoanPhase,
    ) -> ProgressiveLoanServiceResult<()>;
    async fn delete_progressive_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanServiceResult<LoanId>;
}

pub struct ProgressiveLoanServiceImpl<R: ProgressiveLoanRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: ProgressiveLoanRepository> ProgressiveLoanServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: ProgressiveLoanRepository> ProgressiveLoanService for ProgressiveLoanServiceImpl<R> {
    async fn create_progressive_loan(
        &self,
        context: &AuthorizedContext,
        loan: ProgressiveLoan,
    ) -> ProgressiveLoanServiceResult<ProgressiveLoan> {
        self.auth_service
            .check(context, Permission::create_loan())
            .await
            .map_err(|e| ProgressiveLoanError::Internal {
                detail: e.to_string(),
            })?;

        if loan.tenant_id != context.tenant_id {
            return Err(ProgressiveLoanError::TenantIsolationViolation);
        }

        self.repository.insert(context, &loan).await?;
        Ok(loan)
    }

    async fn get_progressive_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanServiceResult<Option<ProgressiveLoan>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| ProgressiveLoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self.repository.find_by_id(context, id).await?;

        if let Some(ref l) = loan {
            if l.tenant_id != context.tenant_id {
                return Err(ProgressiveLoanError::TenantIsolationViolation);
            }
        }

        Ok(loan)
    }

    async fn list_progressive_loans(
        &self,
        context: &AuthorizedContext,
    ) -> ProgressiveLoanServiceResult<Vec<ProgressiveLoan>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| ProgressiveLoanError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn update_progressive_loan(
        &self,
        context: &AuthorizedContext,
        loan: ProgressiveLoan,
    ) -> ProgressiveLoanServiceResult<ProgressiveLoan> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| ProgressiveLoanError::Internal {
                detail: e.to_string(),
            })?;

        if loan.tenant_id != context.tenant_id {
            return Err(ProgressiveLoanError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, loan.id)
            .await?
            .ok_or(ProgressiveLoanError::NotFound(loan.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(ProgressiveLoanError::TenantIsolationViolation);
        }

        self.repository.update(context, &loan).await?;
        Ok(loan)
    }

    async fn advance_phase(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        phase: ProgressiveLoanPhase,
    ) -> ProgressiveLoanServiceResult<()> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| ProgressiveLoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(ProgressiveLoanError::NotFound(id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(ProgressiveLoanError::TenantIsolationViolation);
        }

        self.repository.update_phase(context, id, phase).await
    }

    async fn delete_progressive_loan(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> ProgressiveLoanServiceResult<LoanId> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| ProgressiveLoanError::Internal {
                detail: e.to_string(),
            })?;

        let loan = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(ProgressiveLoanError::NotFound(id))?;

        if loan.tenant_id != context.tenant_id {
            return Err(ProgressiveLoanError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn ProgressiveLoanService) {}
    }
}
