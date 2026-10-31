//! Charge domain service.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Charge, ChargeError};
use crate::repository::ChargeRepository;
use fineract_core::ids::ChargeId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type ChargeServiceResult<T> = Result<T, ChargeError>;

#[async_trait]
pub trait ChargeService: Send + Sync {
    async fn create_charge(
        &self,
        context: &AuthorizedContext,
        charge: Charge,
    ) -> ChargeServiceResult<Charge>;
    async fn get_charge(
        &self,
        context: &AuthorizedContext,
        id: ChargeId,
    ) -> ChargeServiceResult<Option<Charge>>;
    async fn list_charges(&self, context: &AuthorizedContext) -> ChargeServiceResult<Vec<Charge>>;
    async fn update_charge(
        &self,
        context: &AuthorizedContext,
        charge: Charge,
    ) -> ChargeServiceResult<Charge>;
    async fn delete_charge(
        &self,
        context: &AuthorizedContext,
        id: ChargeId,
    ) -> ChargeServiceResult<ChargeId>;
}

pub struct ChargeServiceImpl<R: ChargeRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: ChargeRepository> ChargeServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: ChargeRepository> ChargeService for ChargeServiceImpl<R> {
    async fn create_charge(
        &self,
        context: &AuthorizedContext,
        charge: Charge,
    ) -> ChargeServiceResult<Charge> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| ChargeError::Internal {
                detail: e.to_string(),
            })?;
        if charge.tenant_id != context.tenant_id {
            return Err(ChargeError::TenantIsolationViolation);
        }
        self.repository.insert(context, &charge).await?;
        Ok(charge)
    }

    async fn get_charge(
        &self,
        context: &AuthorizedContext,
        id: ChargeId,
    ) -> ChargeServiceResult<Option<Charge>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ChargeError::Internal {
                detail: e.to_string(),
            })?;
        let charge = self.repository.find_by_id(context, id).await?;
        if let Some(ref c) = charge {
            if c.tenant_id != context.tenant_id {
                return Err(ChargeError::TenantIsolationViolation);
            }
        }
        Ok(charge)
    }

    async fn list_charges(&self, context: &AuthorizedContext) -> ChargeServiceResult<Vec<Charge>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ChargeError::Internal {
                detail: e.to_string(),
            })?;
        self.repository.find_all_by_tenant(context).await
    }

    async fn update_charge(
        &self,
        context: &AuthorizedContext,
        charge: Charge,
    ) -> ChargeServiceResult<Charge> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| ChargeError::Internal {
                detail: e.to_string(),
            })?;
        if charge.tenant_id != context.tenant_id {
            return Err(ChargeError::TenantIsolationViolation);
        }
        self.repository.update(context, &charge).await?;
        Ok(charge)
    }

    async fn delete_charge(
        &self,
        context: &AuthorizedContext,
        id: ChargeId,
    ) -> ChargeServiceResult<ChargeId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| ChargeError::Internal {
                detail: e.to_string(),
            })?;
        let charge = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(ChargeError::NotFound(id))?;
        if charge.tenant_id != context.tenant_id {
            return Err(ChargeError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn ChargeService) {}
    }
}
