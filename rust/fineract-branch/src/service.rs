//! Branch domain service.
//!
//! The service layer enforces authorization before any data access.
//! Every public method receives an explicit [`AuthorizedContext`] and calls
//! [`AuthorizationService::check`] before mutating or reading data.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Office, OfficeError};
use crate::repository::BranchRepository;
use fineract_core::ids::OfficeId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

/// Result type for branch service operations.
pub type BranchServiceResult<T> = Result<T, OfficeError>;

/// Domain service trait for branch operations.
///
/// All methods enforce RBAC + data-level authorization before touching data.
/// This is the primary IDOR defense: every call checks that the caller's
/// tenant matches the resource's tenant.
#[async_trait]
pub trait BranchService: Send + Sync {
    async fn create_office(
        &self,
        context: &AuthorizedContext,
        office: Office,
    ) -> BranchServiceResult<Office>;

    async fn get_office(
        &self,
        context: &AuthorizedContext,
        id: OfficeId,
    ) -> BranchServiceResult<Option<Office>>;

    async fn list_offices(&self, context: &AuthorizedContext) -> BranchServiceResult<Vec<Office>>;

    async fn update_office(
        &self,
        context: &AuthorizedContext,
        office: Office,
    ) -> BranchServiceResult<Office>;

    async fn delete_office(
        &self,
        context: &AuthorizedContext,
        id: OfficeId,
    ) -> BranchServiceResult<OfficeId>;
}

/// Concrete implementation of [`BranchService`].
pub struct BranchServiceImpl<R: BranchRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: BranchRepository> BranchServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: BranchRepository> BranchService for BranchServiceImpl<R> {
    async fn create_office(
        &self,
        context: &AuthorizedContext,
        office: Office,
    ) -> BranchServiceResult<Office> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| OfficeError::Internal {
                detail: e.to_string(),
            })?;

        if office.tenant_id != context.tenant_id {
            return Err(OfficeError::TenantIsolationViolation);
        }

        if let Some(parent_id) = office.parent_office_id {
            if let Some(parent) = self.repository.find_by_id(context, parent_id).await? {
                if parent.tenant_id != context.tenant_id {
                    return Err(OfficeError::ParentTenantMismatch(parent_id));
                }
            } else {
                return Err(OfficeError::ParentNotFound(parent_id));
            }
        }

        self.repository.insert(context, &office).await?;
        Ok(office)
    }

    async fn get_office(
        &self,
        context: &AuthorizedContext,
        id: OfficeId,
    ) -> BranchServiceResult<Option<Office>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| OfficeError::Internal {
                detail: e.to_string(),
            })?;

        let office = self.repository.find_by_id(context, id).await?;

        if let Some(ref office) = office {
            if office.tenant_id != context.tenant_id {
                return Err(OfficeError::TenantIsolationViolation);
            }
        }

        Ok(office)
    }

    async fn list_offices(&self, context: &AuthorizedContext) -> BranchServiceResult<Vec<Office>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| OfficeError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn update_office(
        &self,
        context: &AuthorizedContext,
        office: Office,
    ) -> BranchServiceResult<Office> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| OfficeError::Internal {
                detail: e.to_string(),
            })?;

        if office.tenant_id != context.tenant_id {
            return Err(OfficeError::TenantIsolationViolation);
        }

        self.repository.update(context, &office).await?;
        Ok(office)
    }

    async fn delete_office(
        &self,
        context: &AuthorizedContext,
        id: OfficeId,
    ) -> BranchServiceResult<OfficeId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| OfficeError::Internal {
                detail: e.to_string(),
            })?;

        let office = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(OfficeError::NotFound(id))?;

        if office.tenant_id != context.tenant_id {
            return Err(OfficeError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn BranchService) {}
    }
}
