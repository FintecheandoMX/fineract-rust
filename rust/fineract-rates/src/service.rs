//! Rates domain service.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Rate, RateError};
use crate::repository::RateRepository;
use fineract_core::ids::RateId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type RateServiceResult<T> = Result<T, RateError>;

#[async_trait]
pub trait RateService: Send + Sync {
    async fn create_rate(&self, context: &AuthorizedContext, rate: Rate)
        -> RateServiceResult<Rate>;
    async fn get_rate(
        &self,
        context: &AuthorizedContext,
        id: RateId,
    ) -> RateServiceResult<Option<Rate>>;
    async fn list_rates(&self, context: &AuthorizedContext) -> RateServiceResult<Vec<Rate>>;
    async fn update_rate(&self, context: &AuthorizedContext, rate: Rate)
        -> RateServiceResult<Rate>;
    async fn delete_rate(
        &self,
        context: &AuthorizedContext,
        id: RateId,
    ) -> RateServiceResult<RateId>;
}

pub struct RateServiceImpl<R: RateRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: RateRepository> RateServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: RateRepository> RateService for RateServiceImpl<R> {
    async fn create_rate(
        &self,
        context: &AuthorizedContext,
        rate: Rate,
    ) -> RateServiceResult<Rate> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| RateError::Internal {
                detail: e.to_string(),
            })?;
        if rate.tenant_id != context.tenant_id {
            return Err(RateError::TenantIsolationViolation);
        }
        self.repository.insert(context, &rate).await?;
        Ok(rate)
    }

    async fn get_rate(
        &self,
        context: &AuthorizedContext,
        id: RateId,
    ) -> RateServiceResult<Option<Rate>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| RateError::Internal {
                detail: e.to_string(),
            })?;
        let rate = self.repository.find_by_id(context, id).await?;
        if let Some(ref r) = rate {
            if r.tenant_id != context.tenant_id {
                return Err(RateError::TenantIsolationViolation);
            }
        }
        Ok(rate)
    }

    async fn list_rates(&self, context: &AuthorizedContext) -> RateServiceResult<Vec<Rate>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| RateError::Internal {
                detail: e.to_string(),
            })?;
        self.repository.find_all_by_tenant(context).await
    }

    async fn update_rate(
        &self,
        context: &AuthorizedContext,
        rate: Rate,
    ) -> RateServiceResult<Rate> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| RateError::Internal {
                detail: e.to_string(),
            })?;
        if rate.tenant_id != context.tenant_id {
            return Err(RateError::TenantIsolationViolation);
        }
        self.repository.update(context, &rate).await?;
        Ok(rate)
    }

    async fn delete_rate(
        &self,
        context: &AuthorizedContext,
        id: RateId,
    ) -> RateServiceResult<RateId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| RateError::Internal {
                detail: e.to_string(),
            })?;
        let rate = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(RateError::NotFound(id))?;
        if rate.tenant_id != context.tenant_id {
            return Err(RateError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn RateService) {}
    }
}
