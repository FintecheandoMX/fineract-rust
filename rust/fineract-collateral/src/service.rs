//! Collateral domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Collateral, CollateralError, CollateralType};
use crate::repository::CollateralRepository;
use fineract_core::ids::CollateralId;
use fineract_core::ids::LoanId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type CollateralServiceResult<T> = Result<T, CollateralError>;

#[async_trait]
pub trait CollateralService: Send + Sync {
    async fn create_collateral(
        &self,
        context: &AuthorizedContext,
        collateral: Collateral,
    ) -> CollateralServiceResult<Collateral>;
    async fn get_collateral(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralServiceResult<Option<Collateral>>;
    async fn list_collaterals(
        &self,
        context: &AuthorizedContext,
    ) -> CollateralServiceResult<Vec<Collateral>>;
    async fn list_collaterals_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> CollateralServiceResult<Vec<Collateral>>;
    async fn update_collateral(
        &self,
        context: &AuthorizedContext,
        collateral: Collateral,
    ) -> CollateralServiceResult<Collateral>;
    async fn delete_collateral(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralServiceResult<CollateralId>;
}

pub struct CollateralServiceImpl<R: CollateralRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: CollateralRepository> CollateralServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: CollateralRepository> CollateralService for CollateralServiceImpl<R> {
    async fn create_collateral(
        &self,
        context: &AuthorizedContext,
        collateral: Collateral,
    ) -> CollateralServiceResult<Collateral> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CollateralError::Internal {
                detail: e.to_string(),
            })?;

        if collateral.tenant_id != context.tenant_id {
            return Err(CollateralError::TenantIsolationViolation);
        }

        self.repository.insert(context, &collateral).await?;
        Ok(collateral)
    }

    async fn get_collateral(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralServiceResult<Option<Collateral>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| CollateralError::Internal {
                detail: e.to_string(),
            })?;

        let collateral = self.repository.find_by_id(context, id).await?;

        if let Some(ref c) = collateral {
            if c.tenant_id != context.tenant_id {
                return Err(CollateralError::TenantIsolationViolation);
            }
        }

        Ok(collateral)
    }

    async fn list_collaterals(
        &self,
        context: &AuthorizedContext,
    ) -> CollateralServiceResult<Vec<Collateral>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| CollateralError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn list_collaterals_by_loan_id(
        &self,
        context: &AuthorizedContext,
        loan_id: LoanId,
    ) -> CollateralServiceResult<Vec<Collateral>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| CollateralError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_by_loan_id(context, loan_id).await
    }

    async fn update_collateral(
        &self,
        context: &AuthorizedContext,
        collateral: Collateral,
    ) -> CollateralServiceResult<Collateral> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CollateralError::Internal {
                detail: e.to_string(),
            })?;

        if collateral.tenant_id != context.tenant_id {
            return Err(CollateralError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, collateral.id)
            .await?
            .ok_or(CollateralError::NotFound(collateral.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(CollateralError::TenantIsolationViolation);
        }

        self.repository.update(context, &collateral).await?;
        Ok(collateral)
    }

    async fn delete_collateral(
        &self,
        context: &AuthorizedContext,
        id: CollateralId,
    ) -> CollateralServiceResult<CollateralId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CollateralError::Internal {
                detail: e.to_string(),
            })?;

        let collateral = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(CollateralError::NotFound(id))?;

        if collateral.tenant_id != context.tenant_id {
            return Err(CollateralError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn CollateralService) {}
    }
}
