//! Guarantor domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Guarantor, GuarantorError};
use crate::repository::GuarantorRepository;
use fineract_core::ids::GuarantorId;
use fineract_core::ids::LoanId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type GuarantorServiceResult<T> = Result<T, GuarantorError>;

#[async_trait]
pub trait GuarantorService: Send + Sync {
    async fn create_guarantor(
        &self,
        context: &AuthorizedContext,
        guarantor: Guarantor,
    ) -> GuarantorServiceResult<Guarantor>;
    async fn get_guarantor(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorServiceResult<Option<Guarantor>>;
    async fn list_guarantors(
        &self,
        context: &AuthorizedContext,
    ) -> GuarantorServiceResult<Vec<Guarantor>>;
    async fn list_guarantors_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> GuarantorServiceResult<Vec<Guarantor>>;
    async fn update_guarantor(
        &self,
        context: &AuthorizedContext,
        guarantor: Guarantor,
    ) -> GuarantorServiceResult<Guarantor>;
    async fn delete_guarantor(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorServiceResult<GuarantorId>;
}

pub struct GuarantorServiceImpl<R: GuarantorRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: GuarantorRepository> GuarantorServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: GuarantorRepository> GuarantorService for GuarantorServiceImpl<R> {
    async fn create_guarantor(
        &self,
        context: &AuthorizedContext,
        guarantor: Guarantor,
    ) -> GuarantorServiceResult<Guarantor> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| GuarantorError::Internal {
                detail: e.to_string(),
            })?;

        if guarantor.tenant_id != context.tenant_id {
            return Err(GuarantorError::TenantIsolationViolation);
        }

        self.repository.insert(context, &guarantor).await?;
        Ok(guarantor)
    }

    async fn get_guarantor(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorServiceResult<Option<Guarantor>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| GuarantorError::Internal {
                detail: e.to_string(),
            })?;

        let guarantor = self.repository.find_by_id(context, id).await?;

        if let Some(ref g) = guarantor {
            if g.tenant_id != context.tenant_id {
                return Err(GuarantorError::TenantIsolationViolation);
            }
        }

        Ok(guarantor)
    }

    async fn list_guarantors(
        &self,
        context: &AuthorizedContext,
    ) -> GuarantorServiceResult<Vec<Guarantor>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| GuarantorError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn list_guarantors_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> GuarantorServiceResult<Vec<Guarantor>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| GuarantorError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_by_loan_id(context, loan_id).await
    }

    async fn update_guarantor(
        &self,
        context: &AuthorizedContext,
        guarantor: Guarantor,
    ) -> GuarantorServiceResult<Guarantor> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| GuarantorError::Internal {
                detail: e.to_string(),
            })?;

        if guarantor.tenant_id != context.tenant_id {
            return Err(GuarantorError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, guarantor.id)
            .await?
            .ok_or(GuarantorError::NotFound(guarantor.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(GuarantorError::TenantIsolationViolation);
        }

        self.repository.update(context, &guarantor).await?;
        Ok(guarantor)
    }

    async fn delete_guarantor(
        &self,
        context: &AuthorizedContext,
        id: GuarantorId,
    ) -> GuarantorServiceResult<GuarantorId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| GuarantorError::Internal {
                detail: e.to_string(),
            })?;

        let guarantor = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(GuarantorError::NotFound(id))?;

        if guarantor.tenant_id != context.tenant_id {
            return Err(GuarantorError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn GuarantorService) {}
    }
}
