//! Datatable domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{
    Datatable, DatatableColumn, DatatableColumnId, DatatableError, DatatableView, DatatableViewId,
};
use crate::repository::DatatableRepository;
use fineract_core::ids::DatatableId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type DatatableServiceResult<T> = Result<T, DatatableError>;

#[async_trait]
pub trait DatatableService: Send + Sync {
    // Datatable
    async fn create_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: Datatable,
    ) -> DatatableServiceResult<Datatable>;
    async fn get_datatable(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableServiceResult<Option<Datatable>>;
    async fn list_datatables(
        &self,
        context: &AuthorizedContext,
    ) -> DatatableServiceResult<Vec<Datatable>>;
    async fn update_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: Datatable,
    ) -> DatatableServiceResult<Datatable>;
    async fn delete_datatable(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableServiceResult<DatatableId>;

    // DatatableColumn
    async fn create_column(
        &self,
        context: &AuthorizedContext,
        column: DatatableColumn,
    ) -> DatatableServiceResult<DatatableColumn>;
    async fn get_column(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableServiceResult<Option<DatatableColumn>>;
    async fn list_columns(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableServiceResult<Vec<DatatableColumn>>;
    async fn update_column(
        &self,
        context: &AuthorizedContext,
        column: DatatableColumn,
    ) -> DatatableServiceResult<DatatableColumn>;
    async fn delete_column(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableServiceResult<DatatableColumnId>;

    // DatatableView
    async fn create_view(
        &self,
        context: &AuthorizedContext,
        view: DatatableView,
    ) -> DatatableServiceResult<DatatableView>;
    async fn get_view(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableServiceResult<Option<DatatableView>>;
    async fn list_views(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableServiceResult<Vec<DatatableView>>;
    async fn update_view(
        &self,
        context: &AuthorizedContext,
        view: DatatableView,
    ) -> DatatableServiceResult<DatatableView>;
    async fn delete_view(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableServiceResult<DatatableViewId>;
}

pub struct DatatableServiceImpl<R: DatatableRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: DatatableRepository> DatatableServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: DatatableRepository> DatatableService for DatatableServiceImpl<R> {
    // --- Datatable ---

    async fn create_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: Datatable,
    ) -> DatatableServiceResult<Datatable> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        if datatable.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository
            .insert_datatable(context, &datatable)
            .await?;
        Ok(datatable)
    }

    async fn get_datatable(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableServiceResult<Option<Datatable>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        let datatable = self.repository.find_datatable_by_id(context, id).await?;

        if let Some(ref d) = datatable {
            if d.tenant_id != context.tenant_id {
                return Err(DatatableError::TenantIsolationViolation);
            }
        }

        Ok(datatable)
    }

    async fn list_datatables(
        &self,
        context: &AuthorizedContext,
    ) -> DatatableServiceResult<Vec<Datatable>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_datatables(context).await
    }

    async fn update_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: Datatable,
    ) -> DatatableServiceResult<Datatable> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        if datatable.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_datatable_by_id(context, datatable.id)
            .await?
            .ok_or(DatatableError::DatatableNotFound(datatable.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository
            .update_datatable(context, &datatable)
            .await?;
        Ok(datatable)
    }

    async fn delete_datatable(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableServiceResult<DatatableId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        let datatable = self
            .repository
            .find_datatable_by_id(context, id)
            .await?
            .ok_or(DatatableError::DatatableNotFound(id))?;

        if datatable.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository.delete_datatable(context, id).await?;
        Ok(id)
    }

    // --- DatatableColumn ---

    async fn create_column(
        &self,
        context: &AuthorizedContext,
        column: DatatableColumn,
    ) -> DatatableServiceResult<DatatableColumn> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        if column.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository.insert_column(context, &column).await?;
        Ok(column)
    }

    async fn get_column(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableServiceResult<Option<DatatableColumn>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        let column = self.repository.find_column_by_id(context, id).await?;

        if let Some(ref c) = column {
            if c.tenant_id != context.tenant_id {
                return Err(DatatableError::TenantIsolationViolation);
            }
        }

        Ok(column)
    }

    async fn list_columns(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableServiceResult<Vec<DatatableColumn>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        self.repository
            .find_columns_by_datatable_id(context, datatable_id)
            .await
    }

    async fn update_column(
        &self,
        context: &AuthorizedContext,
        column: DatatableColumn,
    ) -> DatatableServiceResult<DatatableColumn> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        if column.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_column_by_id(context, column.id)
            .await?
            .ok_or(DatatableError::ColumnNotFound(column.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository.update_column(context, &column).await?;
        Ok(column)
    }

    async fn delete_column(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableServiceResult<DatatableColumnId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        let column = self
            .repository
            .find_column_by_id(context, id)
            .await?
            .ok_or(DatatableError::ColumnNotFound(id))?;

        if column.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository.delete_column(context, id).await?;
        Ok(id)
    }

    // --- DatatableView ---

    async fn create_view(
        &self,
        context: &AuthorizedContext,
        view: DatatableView,
    ) -> DatatableServiceResult<DatatableView> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        if view.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository.insert_view(context, &view).await?;
        Ok(view)
    }

    async fn get_view(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableServiceResult<Option<DatatableView>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        let view = self.repository.find_view_by_id(context, id).await?;

        if let Some(ref v) = view {
            if v.tenant_id != context.tenant_id {
                return Err(DatatableError::TenantIsolationViolation);
            }
        }

        Ok(view)
    }

    async fn list_views(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableServiceResult<Vec<DatatableView>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        self.repository
            .find_views_by_datatable_id(context, datatable_id)
            .await
    }

    async fn update_view(
        &self,
        context: &AuthorizedContext,
        view: DatatableView,
    ) -> DatatableServiceResult<DatatableView> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        if view.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_view_by_id(context, view.id)
            .await?
            .ok_or(DatatableError::ViewNotFound(view.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository.update_view(context, &view).await?;
        Ok(view)
    }

    async fn delete_view(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableServiceResult<DatatableViewId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: e.to_string(),
            })?;

        let view = self
            .repository
            .find_view_by_id(context, id)
            .await?
            .ok_or(DatatableError::ViewNotFound(id))?;

        if view.tenant_id != context.tenant_id {
            return Err(DatatableError::TenantIsolationViolation);
        }

        self.repository.delete_view(context, id).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn DatatableService) {}
    }
}
