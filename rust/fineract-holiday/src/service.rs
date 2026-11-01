//! Holiday domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Holiday, HolidayError};
use crate::repository::HolidayRepository;
use fineract_core::ids::HolidayId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type HolidayServiceResult<T> = Result<T, HolidayError>;

#[async_trait]
pub trait HolidayService: Send + Sync {
    async fn create_holiday(
        &self,
        context: &AuthorizedContext,
        holiday: Holiday,
    ) -> HolidayServiceResult<Holiday>;
    async fn get_holiday(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayServiceResult<Option<Holiday>>;
    async fn list_holidays(
        &self,
        context: &AuthorizedContext,
    ) -> HolidayServiceResult<Vec<Holiday>>;
    async fn get_holidays_by_date(
        &self,
        context: &AuthorizedContext,
        date: chrono::NaiveDate,
    ) -> HolidayServiceResult<Vec<Holiday>>;
    async fn update_holiday(
        &self,
        context: &AuthorizedContext,
        holiday: Holiday,
    ) -> HolidayServiceResult<Holiday>;
    async fn delete_holiday(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayServiceResult<HolidayId>;
}

pub struct HolidayServiceImpl<R: HolidayRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: HolidayRepository> HolidayServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: HolidayRepository> HolidayService for HolidayServiceImpl<R> {
    async fn create_holiday(
        &self,
        context: &AuthorizedContext,
        holiday: Holiday,
    ) -> HolidayServiceResult<Holiday> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| HolidayError::Internal {
                detail: e.to_string(),
            })?;

        if holiday.tenant_id != context.tenant_id {
            return Err(HolidayError::TenantIsolationViolation);
        }

        self.repository.insert(context, &holiday).await?;
        Ok(holiday)
    }

    async fn get_holiday(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayServiceResult<Option<Holiday>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| HolidayError::Internal {
                detail: e.to_string(),
            })?;

        let holiday = self.repository.find_by_id(context, id).await?;

        if let Some(ref h) = holiday {
            if h.tenant_id != context.tenant_id {
                return Err(HolidayError::TenantIsolationViolation);
            }
        }

        Ok(holiday)
    }

    async fn list_holidays(
        &self,
        context: &AuthorizedContext,
    ) -> HolidayServiceResult<Vec<Holiday>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| HolidayError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn get_holidays_by_date(
        &self,
        context: &AuthorizedContext,
        date: chrono::NaiveDate,
    ) -> HolidayServiceResult<Vec<Holiday>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| HolidayError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_by_date(context, date).await
    }

    async fn update_holiday(
        &self,
        context: &AuthorizedContext,
        holiday: Holiday,
    ) -> HolidayServiceResult<Holiday> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| HolidayError::Internal {
                detail: e.to_string(),
            })?;

        if holiday.tenant_id != context.tenant_id {
            return Err(HolidayError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, holiday.id)
            .await?
            .ok_or(HolidayError::NotFound(holiday.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(HolidayError::TenantIsolationViolation);
        }

        self.repository.update(context, &holiday).await?;
        Ok(holiday)
    }

    async fn delete_holiday(
        &self,
        context: &AuthorizedContext,
        id: HolidayId,
    ) -> HolidayServiceResult<HolidayId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| HolidayError::Internal {
                detail: e.to_string(),
            })?;

        let holiday = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(HolidayError::NotFound(id))?;

        if holiday.tenant_id != context.tenant_id {
            return Err(HolidayError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn HolidayService) {}
    }
}
