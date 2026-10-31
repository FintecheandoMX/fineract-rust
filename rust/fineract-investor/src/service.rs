//! Investor domain service.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Investor, InvestorError};
use crate::repository::InvestorRepository;
use fineract_core::ids::InvestorId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type InvestorServiceResult<T> = Result<T, InvestorError>;

#[async_trait]
pub trait InvestorService: Send + Sync {
    async fn create_investor(
        &self,
        context: &AuthorizedContext,
        investor: Investor,
    ) -> InvestorServiceResult<Investor>;
    async fn get_investor(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorServiceResult<Option<Investor>>;
    async fn list_investors(
        &self,
        context: &AuthorizedContext,
    ) -> InvestorServiceResult<Vec<Investor>>;
    async fn update_investor(
        &self,
        context: &AuthorizedContext,
        investor: Investor,
    ) -> InvestorServiceResult<Investor>;
    async fn delete_investor(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorServiceResult<InvestorId>;
}

pub struct InvestorServiceImpl<R: InvestorRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: InvestorRepository> InvestorServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: InvestorRepository> InvestorService for InvestorServiceImpl<R> {
    async fn create_investor(
        &self,
        context: &AuthorizedContext,
        investor: Investor,
    ) -> InvestorServiceResult<Investor> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| InvestorError::Internal {
                detail: e.to_string(),
            })?;
        if investor.tenant_id != context.tenant_id {
            return Err(InvestorError::TenantIsolationViolation);
        }
        self.repository.insert(context, &investor).await?;
        Ok(investor)
    }

    async fn get_investor(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorServiceResult<Option<Investor>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| InvestorError::Internal {
                detail: e.to_string(),
            })?;
        let investor = self.repository.find_by_id(context, id).await?;
        if let Some(ref i) = investor {
            if i.tenant_id != context.tenant_id {
                return Err(InvestorError::TenantIsolationViolation);
            }
        }
        Ok(investor)
    }

    async fn list_investors(
        &self,
        context: &AuthorizedContext,
    ) -> InvestorServiceResult<Vec<Investor>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| InvestorError::Internal {
                detail: e.to_string(),
            })?;
        self.repository.find_all_by_tenant(context).await
    }

    async fn update_investor(
        &self,
        context: &AuthorizedContext,
        investor: Investor,
    ) -> InvestorServiceResult<Investor> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| InvestorError::Internal {
                detail: e.to_string(),
            })?;
        if investor.tenant_id != context.tenant_id {
            return Err(InvestorError::TenantIsolationViolation);
        }
        self.repository.update(context, &investor).await?;
        Ok(investor)
    }

    async fn delete_investor(
        &self,
        context: &AuthorizedContext,
        id: InvestorId,
    ) -> InvestorServiceResult<InvestorId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| InvestorError::Internal {
                detail: e.to_string(),
            })?;
        let investor = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(InvestorError::NotFound(id))?;
        if investor.tenant_id != context.tenant_id {
            return Err(InvestorError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn InvestorService) {}
    }
}
