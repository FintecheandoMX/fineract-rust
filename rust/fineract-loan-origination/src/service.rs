//! Loan origination domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{LoanApplication, LoanApplicationStatus, LoanOriginationError};
use crate::repository::LoanApplicationRepository;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type LoanOriginationServiceResult<T> = Result<T, LoanOriginationError>;

#[async_trait]
pub trait LoanOriginationService: Send + Sync {
    async fn submit_application(
        &self,
        context: &AuthorizedContext,
        application: LoanApplication,
    ) -> LoanOriginationServiceResult<LoanApplication>;
    async fn get_application(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanOriginationServiceResult<Option<LoanApplication>>;
    async fn list_applications(
        &self,
        context: &AuthorizedContext,
    ) -> LoanOriginationServiceResult<Vec<LoanApplication>>;
    async fn list_applications_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanOriginationServiceResult<Vec<LoanApplication>>;
    async fn update_application(
        &self,
        context: &AuthorizedContext,
        application: LoanApplication,
    ) -> LoanOriginationServiceResult<LoanApplication>;
    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanApplicationStatus,
        reviewed_by: Option<fineract_core::ids::StaffId>,
        rejection_reason: Option<String>,
    ) -> LoanOriginationServiceResult<()>;
    async fn delete_application(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanOriginationServiceResult<LoanId>;
}

pub struct LoanOriginationServiceImpl<R: LoanApplicationRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: LoanApplicationRepository> LoanOriginationServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: LoanApplicationRepository> LoanOriginationService for LoanOriginationServiceImpl<R> {
    async fn submit_application(
        &self,
        context: &AuthorizedContext,
        application: LoanApplication,
    ) -> LoanOriginationServiceResult<LoanApplication> {
        self.auth_service
            .check(context, Permission::create_loan())
            .await
            .map_err(|e| LoanOriginationError::Internal {
                detail: e.to_string(),
            })?;

        if application.tenant_id != context.tenant_id {
            return Err(LoanOriginationError::TenantIsolationViolation);
        }

        self.repository.insert(context, &application).await?;
        Ok(application)
    }

    async fn get_application(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanOriginationServiceResult<Option<LoanApplication>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| LoanOriginationError::Internal {
                detail: e.to_string(),
            })?;

        let application = self.repository.find_by_id(context, id).await?;

        if let Some(ref a) = application {
            if a.tenant_id != context.tenant_id {
                return Err(LoanOriginationError::TenantIsolationViolation);
            }
        }

        Ok(application)
    }

    async fn list_applications(
        &self,
        context: &AuthorizedContext,
    ) -> LoanOriginationServiceResult<Vec<LoanApplication>> {
        self.auth_service
            .check(context, Permission::view_loan())
            .await
            .map_err(|e| LoanOriginationError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn list_applications_by_client(
        &self,
        context: &AuthorizedContext,
        client_id: ClientId,
    ) -> LoanOriginationServiceResult<Vec<LoanApplication>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| LoanOriginationError::Internal {
                detail: e.to_string(),
            })?;

        let apps = self
            .repository
            .find_all_by_client(context, client_id)
            .await?;

        Ok(apps
            .into_iter()
            .filter(|a| a.tenant_id == context.tenant_id)
            .collect())
    }

    async fn update_application(
        &self,
        context: &AuthorizedContext,
        application: LoanApplication,
    ) -> LoanOriginationServiceResult<LoanApplication> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanOriginationError::Internal {
                detail: e.to_string(),
            })?;

        if application.tenant_id != context.tenant_id {
            return Err(LoanOriginationError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, application.id)
            .await?
            .ok_or(LoanOriginationError::NotFound(application.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(LoanOriginationError::TenantIsolationViolation);
        }

        if existing.status != LoanApplicationStatus::Submitted {
            return Err(LoanOriginationError::CannotModify);
        }

        self.repository.update(context, &application).await?;
        Ok(application)
    }

    async fn update_status(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
        status: LoanApplicationStatus,
        reviewed_by: Option<fineract_core::ids::StaffId>,
        rejection_reason: Option<String>,
    ) -> LoanOriginationServiceResult<()> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanOriginationError::Internal {
                detail: e.to_string(),
            })?;

        let application = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(LoanOriginationError::NotFound(id))?;

        if application.tenant_id != context.tenant_id {
            return Err(LoanOriginationError::TenantIsolationViolation);
        }

        self.repository
            .update_status(context, id, status, reviewed_by, rejection_reason)
            .await
    }

    async fn delete_application(
        &self,
        context: &AuthorizedContext,
        id: LoanId,
    ) -> LoanOriginationServiceResult<LoanId> {
        self.auth_service
            .check(context, Permission::update_loan())
            .await
            .map_err(|e| LoanOriginationError::Internal {
                detail: e.to_string(),
            })?;

        let application = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(LoanOriginationError::NotFound(id))?;

        if application.tenant_id != context.tenant_id {
            return Err(LoanOriginationError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn LoanOriginationService) {}
    }
}
