//! Loan product domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{LoanProduct, LoanProductError};
use crate::repository::LoanProductRepository;
use fineract_core::ids::LoanProductId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type LoanProductServiceResult<T> = Result<T, LoanProductError>;

#[async_trait]
pub trait LoanProductService: Send + Sync {
    async fn create_loan_product(
        &self,
        context: &AuthorizedContext,
        product: LoanProduct,
    ) -> LoanProductServiceResult<LoanProduct>;
    async fn get_loan_product(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductServiceResult<Option<LoanProduct>>;
    async fn list_loan_products(
        &self,
        context: &AuthorizedContext,
    ) -> LoanProductServiceResult<Vec<LoanProduct>>;
    async fn get_loan_product_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> LoanProductServiceResult<Option<LoanProduct>>;
    async fn update_loan_product(
        &self,
        context: &AuthorizedContext,
        product: LoanProduct,
    ) -> LoanProductServiceResult<LoanProduct>;
    async fn delete_loan_product(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductServiceResult<LoanProductId>;
}

pub struct LoanProductServiceImpl<R: LoanProductRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: LoanProductRepository> LoanProductServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: LoanProductRepository> LoanProductService for LoanProductServiceImpl<R> {
    async fn create_loan_product(
        &self,
        context: &AuthorizedContext,
        product: LoanProduct,
    ) -> LoanProductServiceResult<LoanProduct> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| LoanProductError::Internal {
                detail: e.to_string(),
            })?;

        if product.tenant_id != context.tenant_id {
            return Err(LoanProductError::TenantIsolationViolation);
        }

        self.repository.insert(context, &product).await?;
        Ok(product)
    }

    async fn get_loan_product(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductServiceResult<Option<LoanProduct>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| LoanProductError::Internal {
                detail: e.to_string(),
            })?;

        let product = self.repository.find_by_id(context, id).await?;

        if let Some(ref p) = product {
            if p.tenant_id != context.tenant_id {
                return Err(LoanProductError::TenantIsolationViolation);
            }
        }

        Ok(product)
    }

    async fn list_loan_products(
        &self,
        context: &AuthorizedContext,
    ) -> LoanProductServiceResult<Vec<LoanProduct>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| LoanProductError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn get_loan_product_by_name(
        &self,
        context: &AuthorizedContext,
        name: &str,
    ) -> LoanProductServiceResult<Option<LoanProduct>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| LoanProductError::Internal {
                detail: e.to_string(),
            })?;

        let product = self.repository.find_by_name(context, name).await?;

        if let Some(ref p) = product {
            if p.tenant_id != context.tenant_id {
                return Err(LoanProductError::TenantIsolationViolation);
            }
        }

        Ok(product)
    }

    async fn update_loan_product(
        &self,
        context: &AuthorizedContext,
        product: LoanProduct,
    ) -> LoanProductServiceResult<LoanProduct> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| LoanProductError::Internal {
                detail: e.to_string(),
            })?;

        if product.tenant_id != context.tenant_id {
            return Err(LoanProductError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, product.id)
            .await?
            .ok_or(LoanProductError::NotFound(product.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(LoanProductError::TenantIsolationViolation);
        }

        self.repository.update(context, &product).await?;
        Ok(product)
    }

    async fn delete_loan_product(
        &self,
        context: &AuthorizedContext,
        id: LoanProductId,
    ) -> LoanProductServiceResult<LoanProductId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| LoanProductError::Internal {
                detail: e.to_string(),
            })?;

        let product = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(LoanProductError::NotFound(id))?;

        if product.tenant_id != context.tenant_id {
            return Err(LoanProductError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn LoanProductService) {}
    }
}
