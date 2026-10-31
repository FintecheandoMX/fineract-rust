//! Tax domain service.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Tax, TaxError};
use crate::repository::TaxRepository;
use fineract_core::ids::TaxId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type TaxServiceResult<T> = Result<T, TaxError>;

#[async_trait]
pub trait TaxService: Send + Sync {
    async fn create_tax(&self, context: &AuthorizedContext, tax: Tax) -> TaxServiceResult<Tax>;
    async fn get_tax(
        &self,
        context: &AuthorizedContext,
        id: TaxId,
    ) -> TaxServiceResult<Option<Tax>>;
    async fn list_taxes(&self, context: &AuthorizedContext) -> TaxServiceResult<Vec<Tax>>;
    async fn update_tax(&self, context: &AuthorizedContext, tax: Tax) -> TaxServiceResult<Tax>;
    async fn delete_tax(&self, context: &AuthorizedContext, id: TaxId) -> TaxServiceResult<TaxId>;
}

pub struct TaxServiceImpl<R: TaxRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: TaxRepository> TaxServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: TaxRepository> TaxService for TaxServiceImpl<R> {
    async fn create_tax(&self, context: &AuthorizedContext, tax: Tax) -> TaxServiceResult<Tax> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| TaxError::Internal {
                detail: e.to_string(),
            })?;
        if tax.tenant_id != context.tenant_id {
            return Err(TaxError::TenantIsolationViolation);
        }
        self.repository.insert(context, &tax).await?;
        Ok(tax)
    }

    async fn get_tax(
        &self,
        context: &AuthorizedContext,
        id: TaxId,
    ) -> TaxServiceResult<Option<Tax>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| TaxError::Internal {
                detail: e.to_string(),
            })?;
        let tax = self.repository.find_by_id(context, id).await?;
        if let Some(ref t) = tax {
            if t.tenant_id != context.tenant_id {
                return Err(TaxError::TenantIsolationViolation);
            }
        }
        Ok(tax)
    }

    async fn list_taxes(&self, context: &AuthorizedContext) -> TaxServiceResult<Vec<Tax>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| TaxError::Internal {
                detail: e.to_string(),
            })?;
        self.repository.find_all_by_tenant(context).await
    }

    async fn update_tax(&self, context: &AuthorizedContext, tax: Tax) -> TaxServiceResult<Tax> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| TaxError::Internal {
                detail: e.to_string(),
            })?;
        if tax.tenant_id != context.tenant_id {
            return Err(TaxError::TenantIsolationViolation);
        }
        self.repository.update(context, &tax).await?;
        Ok(tax)
    }

    async fn delete_tax(&self, context: &AuthorizedContext, id: TaxId) -> TaxServiceResult<TaxId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| TaxError::Internal {
                detail: e.to_string(),
            })?;
        let tax = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(TaxError::NotFound(id))?;
        if tax.tenant_id != context.tenant_id {
            return Err(TaxError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn TaxService) {}
    }
}
