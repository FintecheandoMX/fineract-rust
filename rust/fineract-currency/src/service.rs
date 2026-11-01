//! Currency domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Currency, CurrencyError};
use crate::repository::CurrencyRepository;
use fineract_core::ids::CurrencyId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type CurrencyServiceResult<T> = Result<T, CurrencyError>;

#[async_trait]
pub trait CurrencyService: Send + Sync {
    async fn create_currency(
        &self,
        context: &AuthorizedContext,
        currency: Currency,
    ) -> CurrencyServiceResult<Currency>;
    async fn get_currency(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyServiceResult<Option<Currency>>;
    async fn list_currencies(
        &self,
        context: &AuthorizedContext,
    ) -> CurrencyServiceResult<Vec<Currency>>;
    async fn get_currency_by_code(
        &self,
        context: &AuthorizedContext,
        code: &str,
    ) -> CurrencyServiceResult<Option<Currency>>;
    async fn update_currency(
        &self,
        context: &AuthorizedContext,
        currency: Currency,
    ) -> CurrencyServiceResult<Currency>;
    async fn delete_currency(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyServiceResult<CurrencyId>;
}

pub struct CurrencyServiceImpl<R: CurrencyRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: CurrencyRepository> CurrencyServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: CurrencyRepository> CurrencyService for CurrencyServiceImpl<R> {
    async fn create_currency(
        &self,
        context: &AuthorizedContext,
        currency: Currency,
    ) -> CurrencyServiceResult<Currency> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CurrencyError::Internal {
                detail: e.to_string(),
            })?;

        if currency.tenant_id != context.tenant_id {
            return Err(CurrencyError::TenantIsolationViolation);
        }

        self.repository.insert(context, &currency).await?;
        Ok(currency)
    }

    async fn get_currency(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyServiceResult<Option<Currency>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CurrencyError::Internal {
                detail: e.to_string(),
            })?;

        let currency = self.repository.find_by_id(context, id).await?;

        if let Some(ref c) = currency {
            if c.tenant_id != context.tenant_id {
                return Err(CurrencyError::TenantIsolationViolation);
            }
        }

        Ok(currency)
    }

    async fn list_currencies(
        &self,
        context: &AuthorizedContext,
    ) -> CurrencyServiceResult<Vec<Currency>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CurrencyError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn get_currency_by_code(
        &self,
        context: &AuthorizedContext,
        code: &str,
    ) -> CurrencyServiceResult<Option<Currency>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CurrencyError::Internal {
                detail: e.to_string(),
            })?;

        let currency = self.repository.find_by_code(context, code).await?;

        if let Some(ref c) = currency {
            if c.tenant_id != context.tenant_id {
                return Err(CurrencyError::TenantIsolationViolation);
            }
        }

        Ok(currency)
    }

    async fn update_currency(
        &self,
        context: &AuthorizedContext,
        currency: Currency,
    ) -> CurrencyServiceResult<Currency> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CurrencyError::Internal {
                detail: e.to_string(),
            })?;

        if currency.tenant_id != context.tenant_id {
            return Err(CurrencyError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, currency.id)
            .await?
            .ok_or(CurrencyError::NotFound(currency.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(CurrencyError::TenantIsolationViolation);
        }

        self.repository.update(context, &currency).await?;
        Ok(currency)
    }

    async fn delete_currency(
        &self,
        context: &AuthorizedContext,
        id: CurrencyId,
    ) -> CurrencyServiceResult<CurrencyId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CurrencyError::Internal {
                detail: e.to_string(),
            })?;

        let currency = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(CurrencyError::NotFound(id))?;

        if currency.tenant_id != context.tenant_id {
            return Err(CurrencyError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn CurrencyService) {}
    }
}
