//! Accounting domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Account, AccountingError};
use crate::repository::AccountRepository;
use fineract_core::ids::AccountId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type AccountingServiceResult<T> = Result<T, AccountingError>;

#[async_trait]
pub trait AccountService: Send + Sync {
    async fn create_account(
        &self,
        context: &AuthorizedContext,
        account: Account,
    ) -> AccountingServiceResult<Account>;
    async fn get_account(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingServiceResult<Option<Account>>;
    async fn list_accounts(
        &self,
        context: &AuthorizedContext,
    ) -> AccountingServiceResult<Vec<Account>>;
    async fn update_account(
        &self,
        context: &AuthorizedContext,
        account: Account,
    ) -> AccountingServiceResult<Account>;
    async fn delete_account(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingServiceResult<AccountId>;
}

pub struct AccountServiceImpl<R: AccountRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: AccountRepository> AccountServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: AccountRepository> AccountService for AccountServiceImpl<R> {
    async fn create_account(
        &self,
        context: &AuthorizedContext,
        account: Account,
    ) -> AccountingServiceResult<Account> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| AccountingError::Internal {
                detail: e.to_string(),
            })?;

        if account.tenant_id != context.tenant_id {
            return Err(AccountingError::TenantIsolationViolation);
        }

        self.repository.insert(context, &account).await?;
        Ok(account)
    }

    async fn get_account(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingServiceResult<Option<Account>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| AccountingError::Internal {
                detail: e.to_string(),
            })?;

        let account = self.repository.find_by_id(context, id).await?;

        if let Some(ref a) = account {
            if a.tenant_id != context.tenant_id {
                return Err(AccountingError::TenantIsolationViolation);
            }
        }

        Ok(account)
    }

    async fn list_accounts(
        &self,
        context: &AuthorizedContext,
    ) -> AccountingServiceResult<Vec<Account>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| AccountingError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn update_account(
        &self,
        context: &AuthorizedContext,
        account: Account,
    ) -> AccountingServiceResult<Account> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| AccountingError::Internal {
                detail: e.to_string(),
            })?;

        if account.tenant_id != context.tenant_id {
            return Err(AccountingError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, account.id)
            .await?
            .ok_or(AccountingError::NotFound(account.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(AccountingError::TenantIsolationViolation);
        }

        self.repository.update(context, &account).await?;
        Ok(account)
    }

    async fn delete_account(
        &self,
        context: &AuthorizedContext,
        id: AccountId,
    ) -> AccountingServiceResult<AccountId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| AccountingError::Internal {
                detail: e.to_string(),
            })?;

        let account = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(AccountingError::NotFound(id))?;

        if account.tenant_id != context.tenant_id {
            return Err(AccountingError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn AccountService) {}
    }
}
