//! Savings domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{SavingsAccount, SavingsError};
use crate::repository::SavingsAccountRepository;
use fineract_core::ids::SavingsAccountId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type SavingsServiceResult<T> = Result<T, SavingsError>;

#[async_trait]
pub trait SavingsAccountService: Send + Sync {
    async fn create_account(
        &self,
        context: &AuthorizedContext,
        account: SavingsAccount,
    ) -> SavingsServiceResult<SavingsAccount>;
    async fn get_account(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsServiceResult<Option<SavingsAccount>>;
    async fn list_accounts(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsServiceResult<Vec<SavingsAccount>>;
    async fn update_account(
        &self,
        context: &AuthorizedContext,
        account: SavingsAccount,
    ) -> SavingsServiceResult<SavingsAccount>;
    async fn delete_account(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsServiceResult<SavingsAccountId>;
    async fn activate_account(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsServiceResult<SavingsAccountId>;
}

pub struct SavingsAccountServiceImpl<R: SavingsAccountRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: SavingsAccountRepository> SavingsAccountServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: SavingsAccountRepository> SavingsAccountService for SavingsAccountServiceImpl<R> {
    async fn create_account(
        &self,
        context: &AuthorizedContext,
        account: SavingsAccount,
    ) -> SavingsServiceResult<SavingsAccount> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| SavingsError::Internal {
                detail: e.to_string(),
            })?;

        if account.tenant_id != context.tenant_id {
            return Err(SavingsError::TenantIsolationViolation);
        }

        self.repository.insert(context, &account).await?;
        Ok(account)
    }

    async fn get_account(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsServiceResult<Option<SavingsAccount>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| SavingsError::Internal {
                detail: e.to_string(),
            })?;

        let account = self.repository.find_by_id(context, id).await?;

        if let Some(ref a) = account {
            if a.tenant_id != context.tenant_id {
                return Err(SavingsError::TenantIsolationViolation);
            }
        }

        Ok(account)
    }

    async fn list_accounts(
        &self,
        context: &AuthorizedContext,
    ) -> SavingsServiceResult<Vec<SavingsAccount>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| SavingsError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn update_account(
        &self,
        context: &AuthorizedContext,
        account: SavingsAccount,
    ) -> SavingsServiceResult<SavingsAccount> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| SavingsError::Internal {
                detail: e.to_string(),
            })?;

        if account.tenant_id != context.tenant_id {
            return Err(SavingsError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, account.id)
            .await?
            .ok_or(SavingsError::NotFound(account.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(SavingsError::TenantIsolationViolation);
        }

        self.repository.update(context, &account).await?;
        Ok(account)
    }

    async fn delete_account(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsServiceResult<SavingsAccountId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| SavingsError::Internal {
                detail: e.to_string(),
            })?;

        let account = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(SavingsError::NotFound(id))?;

        if account.tenant_id != context.tenant_id {
            return Err(SavingsError::TenantIsolationViolation);
        }

        self.repository.delete(context, id).await?;
        Ok(id)
    }

    async fn activate_account(
        &self,
        context: &AuthorizedContext,
        id: SavingsAccountId,
    ) -> SavingsServiceResult<SavingsAccountId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| SavingsError::Internal {
                detail: e.to_string(),
            })?;

        let mut account = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(SavingsError::NotFound(id))?;

        if account.tenant_id != context.tenant_id {
            return Err(SavingsError::TenantIsolationViolation);
        }

        account.status = crate::domain::SavingsAccountStatus::Active;
        account.active = true;
        self.repository.update(context, &account).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn SavingsAccountService) {}
    }
}
