//! Authorized security context.
//!
//! Every request carries an explicit [`AuthorizedContext`]. It is passed as a
//! parameter to every domain service and repository — never read from ambient
//! / ThreadLocal state that a caller could forget to set. This makes
//! authorization a compile-time requirement.

use crate::errors::SecurityError;
use fineract_core::ids::*;
use std::collections::HashSet;
/// The authenticated principal (user / service account) making a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub app_user_id: AppUserId,
    pub username: String,
    pub roles: HashSet<RoleId>,
    pub permissions: HashSet<PermissionId>,
    pub office_id: Option<OfficeId>,
}

/// Request-scoped security context. Carries tenant + principal + session.
#[derive(Debug, Clone)]
pub struct AuthorizedContext {
    pub tenant_id: TenantId,
    pub principal: Principal,
    pub session_id: SessionId,
    pub authenticated_at: chrono::DateTime<chrono::Utc>,
}

impl AuthorizedContext {
    pub fn new(
        tenant_id: TenantId,
        principal: Principal,
        session_id: SessionId,
        authenticated_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            tenant_id,
            principal,
            session_id,
            authenticated_at,
        }
    }

    /// Returns true if this context holds the given permission.
    pub fn has_permission(&self, permission: PermissionId) -> bool {
        self.principal.permissions.contains(&permission)
    }

    /// Returns true if this context holds any of the given roles.
    pub fn has_any_role(&self, roles: &[RoleId]) -> bool {
        roles.iter().any(|r| self.principal.roles.contains(r))
    }
}

/// Opaque session identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub uuid::Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Tenant-scoped data access guard. Returned by the authorization service after
/// a successful check; carries the tenant_id that all subsequent queries must
/// use (prevents tenant leakage).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantGuard {
    pub tenant_id: TenantId,
}

impl TenantGuard {
    pub fn new(tenant_id: TenantId) -> Self {
        Self { tenant_id }
    }
}

/// Authorization outcome returned by [`AuthorizationService::check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationDecision {
    Allowed(TenantGuard),
    Denied(SecurityError),
}

impl AuthorizationDecision {
    pub fn require(self) -> Result<TenantGuard, SecurityError> {
        match self {
            Self::Allowed(guard) => Ok(guard),
            Self::Denied(err) => Err(err),
        }
    }
}
