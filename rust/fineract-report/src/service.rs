//! Report domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Report, ReportError, ReportFormat, ReportStatus, ReportType};
use crate::repository::ReportRepository;
use fineract_core::ids::ReportId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type ReportServiceResult<T> = Result<T, ReportError>;

#[async_trait]
pub trait ReportService: Send + Sync {
    async fn create_report(
        &self,
        context: &AuthorizedContext,
        report: Report,
    ) -> ReportServiceResult<Report>;
    async fn get_report(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportServiceResult<Option<Report>>;
    async fn list_reports(&self, context: &AuthorizedContext) -> ReportServiceResult<Vec<Report>>;
    async fn list_reports_by_status(
        &self,
        context: &AuthorizedContext,
        status: ReportStatus,
    ) -> ReportServiceResult<Vec<Report>>;
    async fn update_report(
        &self,
        context: &AuthorizedContext,
        report: Report,
    ) -> ReportServiceResult<Report>;
    async fn delete_report(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportServiceResult<ReportId>;
    async fn mark_report_generating(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportServiceResult<Report>;
    async fn mark_report_completed(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
        file_path: String,
    ) -> ReportServiceResult<Report>;
    async fn mark_report_failed(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
        error_message: String,
    ) -> ReportServiceResult<Report>;
}

pub struct ReportServiceImpl<R: ReportRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: ReportRepository> ReportServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: ReportRepository> ReportService for ReportServiceImpl<R> {
    async fn create_report(
        &self,
        context: &AuthorizedContext,
        report: Report,
    ) -> ReportServiceResult<Report> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| ReportError::Internal {
                detail: e.to_string(),
            })?;

        if report.tenant_id != context.tenant_id {
            return Err(ReportError::TenantIsolationViolation);
        }

        self.repository.insert(context, &report).await?;
        Ok(report)
    }

    async fn get_report(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportServiceResult<Option<Report>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ReportError::Internal {
                detail: e.to_string(),
            })?;

        let report = self.repository.find_by_id(context, id).await?;

        if let Some(ref r) = report {
            if r.tenant_id != context.tenant_id {
                return Err(ReportError::TenantIsolationViolation);
            }
        }

        Ok(report)
    }

    async fn list_reports(&self, context: &AuthorizedContext) -> ReportServiceResult<Vec<Report>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ReportError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn list_reports_by_status(
        &self,
        context: &AuthorizedContext,
        status: ReportStatus,
    ) -> ReportServiceResult<Vec<Report>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ReportError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_by_status(context, status).await
    }

    async fn update_report(
        &self,
        context: &AuthorizedContext,
        report: Report,
    ) -> ReportServiceResult<Report> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| ReportError::Internal {
                detail: e.to_string(),
            })?;

        if report.tenant_id != context.tenant_id {
            return Err(ReportError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, report.id)
            .await?
            .ok_or(ReportError::NotFound(report.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(ReportError::TenantIsolationViolation);
        }

        self.repository.update(context, &report).await?;
        Ok(report)
    }

    async fn delete_report(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportServiceResult<ReportId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| ReportError::Internal {
                detail: e.to_string(),
            })?;

        let report = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(ReportError::NotFound(id))?;

        if report.tenant_id != context.tenant_id {
            return Err(ReportError::TenantIsolationViolation);
        }

        self.repository.delete(context, id).await?;
        Ok(id)
    }

    async fn mark_report_generating(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
    ) -> ReportServiceResult<Report> {
        let mut report = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(ReportError::NotFound(id))?;

        if report.tenant_id != context.tenant_id {
            return Err(ReportError::TenantIsolationViolation);
        }

        report.status = crate::domain::ReportStatus::Generating;
        report.generated_by = Some(context.principal.app_user_id.value());
        report.generated_at = Some(chrono::Utc::now());
        report.updated_at = Some(chrono::Utc::now());

        self.repository.update(context, &report).await?;
        Ok(report)
    }

    async fn mark_report_completed(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
        file_path: String,
    ) -> ReportServiceResult<Report> {
        let mut report = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(ReportError::NotFound(id))?;

        if report.tenant_id != context.tenant_id {
            return Err(ReportError::TenantIsolationViolation);
        }

        report.status = crate::domain::ReportStatus::Completed;
        report.file_path = Some(file_path);
        report.updated_at = Some(chrono::Utc::now());

        self.repository.update(context, &report).await?;
        Ok(report)
    }

    async fn mark_report_failed(
        &self,
        context: &AuthorizedContext,
        id: ReportId,
        error_message: String,
    ) -> ReportServiceResult<Report> {
        let mut report = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(ReportError::NotFound(id))?;

        if report.tenant_id != context.tenant_id {
            return Err(ReportError::TenantIsolationViolation);
        }

        report.status = crate::domain::ReportStatus::Failed;
        report.error_message = Some(error_message);
        report.updated_at = Some(chrono::Utc::now());

        self.repository.update(context, &report).await?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn ReportService) {}
    }
}
