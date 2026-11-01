//! Savings product domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{SavingsProduct, SavingsProductError};
use crate::repository::SavingsProductRepository;
use fineract_core::ids::SavingsProductId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type SavingsProductServiceResult<T> = Result<T, SavingsProductError>;

#[async_trait]
pub trait SavingsProductService: Send + Sync {
    async fn create_savings_product(
        &self,
        context: &AuthorizedContext,
        product: SavingsProduct,
    ) -> SavingsProductServiceResult<SavingsProduct>;
    async fn get_savings_product(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductServiceResult<Option<SavingsProduct>>;
    async fn list_savings_products(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsProductServiceResult<Vec<SavingsProduct>>;
    async fn get_savings_product_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> SavingsProductServiceResult<Option<SavingsProduct>>;
    async fn update_savings_product(
        &self,
        context: &AuthorizedContext,
        product: SavingsProduct,
    ) -> SavingsProductServiceResult<SavingsProduct>;
    async fn delete_savings_product(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductServiceResult<SavingsProductId>;
}

pub struct SavingsProductServiceImpl<R: SavingsProductRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: SavingsProductRepository> SavingsProductServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: SavingsProductRepository> SavingsProductService for SavingsProductServiceImpl<R> {
    async fn create_savings_product(
        &self,
        context: &AuthorizedContext,
        product: SavingsProduct,
    ) -> SavingsProductServiceResult<SavingsProduct> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| SavingsProductError::Internal {
                detail: e.to_string(),
            })?;

        if product.tenant_id != context.tenant_id {
            return Err(SavingsProductError::TenantIsolationViolation);
        }

        self.repository.insert(context, &product).await?;
        Ok(product)
    }

    async fn get_savings_product(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductServiceResult<Option<SavingsProduct>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| SavingsProductError::Internal {
                detail: e.to_string(),
            })?;

        let product = self.repository.find_by_id(context, id).await?;

        if let Some(ref p) = product {
            if p.tenant_id != context.tenant_id {
                return Err(SavingsProductError::TenantIsolationViolation);
            }
        }

        Ok(product)
    }

    async fn list_savings_products(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsProductServiceResult<Vec<SavingsProduct>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| SavingsProductError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn get_savings_product_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> SavingsProductServiceResult<Option<SavingsProduct>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| SavingsProductError::Internal {
                detail: e.to_string(),
            })?;

        let product = self.repository.find_by_name(context, name).await?;

        if let Some(ref p) = product {
            if p.tenant_id != context.tenant_id {
                return Err(SavingsProductError::TenantIsolationViolation);
            }
        }

        Ok(product)
    }

    async fn update_savings_product(
        &self,
        context: &AuthorizedContext,
        product: SavingsProduct,
    ) -> SavingsProductServiceResult<SavingsProduct> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| SavingsProductError::Internal {
                detail: e.to_string(),
            })?;

        if product.tenant_id != context.tenant_id {
            return Err(SavingsProductError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, product.id)
            .await?
            .ok_or(SavingsProductError::NotFound(product.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(SavingsProductError::TenantIsolationViolation);
        }

        self.repository.update(context, &product).await?;
        Ok(product)
    }

    async fn delete_savings_product(
        &self,
        context: &AuthorizedContext,
        id: SavingsProductId,
    ) -> SavingsProductServiceResult<SavingsProductId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| SavingsProductError::Internal {
                detail: e.to_string(),
            })?;

        let product = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(SavingsProductError::NotFound(id))?;

        if product.tenant_id != context.tenant_id {
            return Err(SavingsProductError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn SavingsProductService) {}
    }
}
