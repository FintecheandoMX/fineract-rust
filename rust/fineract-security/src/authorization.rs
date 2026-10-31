//! Authorization service: RBAC + data-level checks.
//!
//! [`AuthorizationService`] is the mandatory gate for every data access.
//! Domain services must call `check` before reading or mutating any resource.
//! [`check_resource_owner`] enforces IDOR by verifying the requested resource
//! belongs to the caller's tenant / office / client hierarchy.

use async_trait::async_trait;

use crate::context::{AuthorizedContext, TenantGuard};
use crate::errors::SecurityError;
use fineract_core::ids::*;
use std::collections::HashMap;

/// A permission (e.g. "CREATE_LOAN", "VIEW_CLIENT", "POST_JOURNAL_ENTRY").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Permission(pub PermissionId);

impl Permission {
    pub fn create_loan() -> Permission {
        Permission(PermissionId::new(1001))
    }
    pub fn view_loan() -> Permission {
        Permission(PermissionId::new(1002))
    }
    pub fn update_loan() -> Permission {
        Permission(PermissionId::new(1003))
    }
    pub fn delete_loan() -> Permission {
        Permission(PermissionId::new(1004))
    }
    pub fn create_client() -> Permission {
        Permission(PermissionId::new(2001))
    }
    pub fn view_client() -> Permission {
        Permission(PermissionId::new(2002))
    }
    pub fn update_client() -> Permission {
        Permission(PermissionId::new(2003))
    }
    pub fn delete_client() -> Permission {
        Permission(PermissionId::new(2004))
    }
    pub fn create_savings_account() -> Permission {
        Permission(PermissionId::new(3001))
    }
    pub fn view_savings_account() -> Permission {
        Permission(PermissionId::new(3002))
    }
    pub fn post_journal_entry() -> Permission {
        Permission(PermissionId::new(4001))
    }
    pub fn view_journal_entry() -> Permission {
        Permission(PermissionId::new(4002))
    }
    pub fn manage_users() -> Permission {
        Permission(PermissionId::new(5001))
    }
    pub fn manage_tenants() -> Permission {
        Permission(PermissionId::new(6001))
    }
}

#[async_trait]
pub trait AuthorizationService: Send + Sync {
    /// Check whether the context is authorized for the given permission within
    /// its tenant. Returns a [`TenantGuard`] on success (carrying the verified
    /// tenant_id) or a [`SecurityError`] on denial.
    async fn check(
        &self,
        context: &AuthorizedContext,
        permission: Permission,
    ) -> Result<TenantGuard, SecurityError>;

    /// IDOR check: verify that the requested resource (identified by its owning
    /// tenant_id) is accessible by the context. Returns [`TenantGuard`] if the
    /// context's tenant matches the resource's tenant; [`SecurityError`]
    /// otherwise. This is the primary IDOR defense — call it before any
    /// resource read/write.
    async fn check_resource_owner(
        &self,
        context: &AuthorizedContext,
        resource_tenant_id: TenantId,
    ) -> Result<TenantGuard, SecurityError>;

    /// Data-level office check: verify the context's office (if any) may access
    /// the resource's office.
    async fn check_office_access(
        &self,
        context: &AuthorizedContext,
        resource_office_id: OfficeId,
    ) -> Result<TenantGuard, SecurityError>;
}

/// Default in-memory RBAC authorization service. Production implementations
/// should back onto the permission/role/office tables.
pub struct InMemoryAuthorizationService {
    // role_id -> set of permissions
    pub role_permissions: HashMap<RoleId, Vec<PermissionId>>,
    // office_id -> set of accessible office ids (same-office + children)
    pub office_access: HashMap<OfficeId, Vec<OfficeId>>,
}

impl Default for InMemoryAuthorizationService {
    fn default() -> Self {
        let mut role_permissions: HashMap<RoleId, Vec<PermissionId>> = HashMap::new();
        // teller role: read loans/clients/savings
        role_permissions.insert(
            RoleId::new(1),
            vec![
                Permission::view_loan().0,
                Permission::view_client().0,
                Permission::view_savings_account().0,
                Permission::view_journal_entry().0,
            ],
        );
        // loan officer role: create/update loans + view clients
        role_permissions.insert(
            RoleId::new(2),
            vec![
                Permission::create_loan().0,
                Permission::update_loan().0,
                Permission::view_loan().0,
                Permission::view_client().0,
            ],
        );
        // admin role: everything
        role_permissions.insert(
            RoleId::new(3),
            vec![
                Permission::create_loan().0,
                Permission::view_loan().0,
                Permission::update_loan().0,
                Permission::delete_loan().0,
                Permission::create_client().0,
                Permission::view_client().0,
                Permission::create_savings_account().0,
                Permission::view_savings_account().0,
                Permission::post_journal_entry().0,
                Permission::view_journal_entry().0,
                Permission::manage_users().0,
                Permission::manage_tenants().0,
            ],
        );

        let mut office_access: HashMap<OfficeId, Vec<OfficeId>> = HashMap::new();
        office_access.insert(OfficeId::new(1), vec![OfficeId::new(1), OfficeId::new(2)]);
        office_access.insert(OfficeId::new(2), vec![OfficeId::new(2)]);

        Self {
            role_permissions,
            office_access,
        }
    }
}

#[async_trait]
impl AuthorizationService for InMemoryAuthorizationService {
    async fn check(
        &self,
        context: &AuthorizedContext,
        permission: Permission,
    ) -> Result<TenantGuard, SecurityError> {
        // Tenant must always be present (multi-tenancy isolation).
        let _ = context.tenant_id;

        // RBAC: does any of the principal's roles grant this permission?
        let allowed = context
            .principal
            .roles
            .iter()
            .flat_map(|r| self.role_permissions.get(r))
            .flatten()
            .any(|p| *p == permission.0)
            // Direct permission grants also count.
            || context.principal.permissions.contains(&permission.0);

        if !allowed {
            return Err(SecurityError::InsufficientPermissions {
                permission: permission.0.to_string(),
            });
        }

        Ok(TenantGuard::new(context.tenant_id))
    }

    async fn check_resource_owner(
        &self,
        context: &AuthorizedContext,
        resource_tenant_id: TenantId,
    ) -> Result<TenantGuard, SecurityError> {
        if context.tenant_id != resource_tenant_id {
            return Err(SecurityError::TenantIsolationViolation);
        }
        Ok(TenantGuard::new(context.tenant_id))
    }

    async fn check_office_access(
        &self,
        context: &AuthorizedContext,
        resource_office_id: OfficeId,
    ) -> Result<TenantGuard, SecurityError> {
        let allowed = context
            .principal
            .office_id
            .and_then(|oid| self.office_access.get(&oid))
            .is_some_and(|accessible| accessible.contains(&resource_office_id));
        if !allowed {
            return Err(SecurityError::AccessDenied {
                detail: "office not accessible from caller's office".into(),
            });
        }
        Ok(TenantGuard::new(context.tenant_id))
    }
}
