//! Staff domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Staff, StaffError};
use crate::repository::StaffRepository;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type StaffServiceResult<T> = Result<T, StaffError>;

#[async_trait]
pub trait StaffService: Send + Sync {
    async fn create_staff(
        &self,
        context: &AuthorizedContext,
        staff: Staff,
    ) -> StaffServiceResult<Staff>;
    async fn get_staff(
        &self,
        context: &AuthorizedContext,
        id: StaffId,
    ) -> StaffServiceResult<Option<Staff>>;
    async fn get_staff_by_username(
        &self,
        context: &AuthorizedContext,
        username: &str,
    ) -> StaffServiceResult<Option<Staff>>;
    async fn list_staff(&self, context: &AuthorizedContext) -> StaffServiceResult<Vec<Staff>>;
    async fn list_staff_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> StaffServiceResult<Vec<Staff>>;
    async fn update_staff(
        &self,
        context: &AuthorizedContext,
        staff: Staff,
    ) -> StaffServiceResult<Staff>;
    async fn delete_staff(
        &self,
        context: &AuthorizedContext,
        id: StaffId,
    ) -> StaffServiceResult<StaffId>;
}

pub struct StaffServiceImpl<R: StaffRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: StaffRepository> StaffServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: StaffRepository> StaffService for StaffServiceImpl<R> {
    async fn create_staff(
        &self,
        context: &AuthorizedContext,
        staff: Staff,
    ) -> StaffServiceResult<Staff> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| StaffError::Internal {
                detail: e.to_string(),
            })?;

        if staff.tenant_id != context.tenant_id {
            return Err(StaffError::TenantIsolationViolation);
        }

        self.repository.insert(context, &staff).await?;
        Ok(staff)
    }

    async fn get_staff(
        &self,
        context: &AuthorizedContext,
        id: StaffId,
    ) -> StaffServiceResult<Option<Staff>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| StaffError::Internal {
                detail: e.to_string(),
            })?;

        let staff = self.repository.find_by_id(context, id).await?;

        if let Some(ref s) = staff {
            if s.tenant_id != context.tenant_id {
                return Err(StaffError::TenantIsolationViolation);
            }
        }

        Ok(staff)
    }

    async fn get_staff_by_username(
        &self,
        context: &AuthorizedContext,
        username: &str,
    ) -> StaffServiceResult<Option<Staff>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| StaffError::Internal {
                detail: e.to_string(),
            })?;

        let staff = self.repository.find_by_username(context, username).await?;

        if let Some(ref s) = staff {
            if s.tenant_id != context.tenant_id {
                return Err(StaffError::TenantIsolationViolation);
            }
        }

        Ok(staff)
    }

    async fn list_staff(&self, context: &AuthorizedContext) -> StaffServiceResult<Vec<Staff>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| StaffError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn list_staff_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> StaffServiceResult<Vec<Staff>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| StaffError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_office(context, office_id).await
    }

    async fn update_staff(
        &self,
        context: &AuthorizedContext,
        staff: Staff,
    ) -> StaffServiceResult<Staff> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| StaffError::Internal {
                detail: e.to_string(),
            })?;

        if staff.tenant_id != context.tenant_id {
            return Err(StaffError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, staff.id)
            .await?
            .ok_or(StaffError::NotFound(staff.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(StaffError::TenantIsolationViolation);
        }

        self.repository.update(context, &staff).await?;
        Ok(staff)
    }

    async fn delete_staff(
        &self,
        context: &AuthorizedContext,
        id: StaffId,
    ) -> StaffServiceResult<StaffId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| StaffError::Internal {
                detail: e.to_string(),
            })?;

        let staff = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(StaffError::NotFound(id))?;

        if staff.tenant_id != context.tenant_id {
            return Err(StaffError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn StaffService) {}
    }
}
