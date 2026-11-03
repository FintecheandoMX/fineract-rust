//! Command handlers for datatable operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{
    Datatable, DatatableColumn, DatatableColumnId, DatatableError, DatatableView, DatatableViewId,
};
use crate::service::DatatableService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::DatatableId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatatableCommand {
    pub tenant_id: TenantId,
    pub registered_table_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub multi_tenant: bool,
}

impl CommandRequest for CreateDatatableCommand {
    type Response = DatatableId;
    const COMMAND_TYPE: &'static str = "CreateDatatable";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatatableCommand {
    pub id: DatatableId,
    pub tenant_id: TenantId,
    pub registered_table_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub multi_tenant: bool,
}

impl CommandRequest for UpdateDatatableCommand {
    type Response = DatatableId;
    const COMMAND_TYPE: &'static str = "UpdateDatatable";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDatatableCommand {
    pub id: DatatableId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteDatatableCommand {
    type Response = DatatableId;
    const COMMAND_TYPE: &'static str = "DeleteDatatable";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatatableColumnCommand {
    pub tenant_id: TenantId,
    pub datatable_id: DatatableId,
    pub column_name: String,
    pub display_name: String,
    pub column_type: String,
    pub column_length: Option<i32>,
    pub column_precision: Option<i32>,
    pub column_scale: Option<i32>,
    pub is_mandatory: bool,
    pub is_unique: bool,
    pub is_foreign_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_column: Option<String>,
}

impl CommandRequest for CreateDatatableColumnCommand {
    type Response = DatatableColumnId;
    const COMMAND_TYPE: &'static str = "CreateDatatableColumn";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatatableColumnCommand {
    pub id: DatatableColumnId,
    pub tenant_id: TenantId,
    pub datatable_id: DatatableId,
    pub column_name: String,
    pub display_name: String,
    pub column_type: String,
    pub column_length: Option<i32>,
    pub column_precision: Option<i32>,
    pub column_scale: Option<i32>,
    pub is_mandatory: bool,
    pub is_unique: bool,
    pub is_foreign_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_column: Option<String>,
}

impl CommandRequest for UpdateDatatableColumnCommand {
    type Response = DatatableColumnId;
    const COMMAND_TYPE: &'static str = "UpdateDatatableColumn";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDatatableColumnCommand {
    pub id: DatatableColumnId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteDatatableColumnCommand {
    type Response = DatatableColumnId;
    const COMMAND_TYPE: &'static str = "DeleteDatatableColumn";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatatableViewCommand {
    pub tenant_id: TenantId,
    pub datatable_id: DatatableId,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<String>,
    pub filter_criteria: Option<serde_json::Value>,
    pub sort_criteria: Option<serde_json::Value>,
}

impl CommandRequest for CreateDatatableViewCommand {
    type Response = DatatableViewId;
    const COMMAND_TYPE: &'static str = "CreateDatatableView";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatatableViewCommand {
    pub id: DatatableViewId,
    pub tenant_id: TenantId,
    pub datatable_id: DatatableId,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<String>,
    pub filter_criteria: Option<serde_json::Value>,
    pub sort_criteria: Option<serde_json::Value>,
}

impl CommandRequest for UpdateDatatableViewCommand {
    type Response = DatatableViewId;
    const COMMAND_TYPE: &'static str = "UpdateDatatableView";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDatatableViewCommand {
    pub id: DatatableViewId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteDatatableViewCommand {
    type Response = DatatableViewId;
    const COMMAND_TYPE: &'static str = "DeleteDatatableView";
}

pub struct CreateDatatableHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> CreateDatatableHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<CreateDatatableCommand> for CreateDatatableHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateDatatable";

    async fn handle(
        &self,
        request: CreateDatatableCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableId, fineract_command::CommandError> {
        let datatable = Datatable::new(
            DatatableId::new(1),
            request.tenant_id,
            request.registered_table_name,
            request.display_name,
            request.description,
            request.multi_tenant,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_datatable(context, datatable)
            .await
            .map(|d| d.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateDatatableHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> UpdateDatatableHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<UpdateDatatableCommand> for UpdateDatatableHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateDatatable";

    async fn handle(
        &self,
        request: UpdateDatatableCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableId, fineract_command::CommandError> {
        let datatable = Datatable::new(
            request.id,
            request.tenant_id,
            request.registered_table_name,
            request.display_name,
            request.description,
            request.multi_tenant,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .update_datatable(context, datatable)
            .await
            .map(|d| d.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteDatatableHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> DeleteDatatableHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<DeleteDatatableCommand> for DeleteDatatableHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteDatatable";

    async fn handle(
        &self,
        request: DeleteDatatableCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableId, fineract_command::CommandError> {
        self.service
            .delete_datatable(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct CreateDatatableColumnHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> CreateDatatableColumnHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<CreateDatatableColumnCommand>
    for CreateDatatableColumnHandler<S>
{
    const COMMAND_TYPE: &'static str = "CreateDatatableColumn";

    async fn handle(
        &self,
        request: CreateDatatableColumnCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableColumnId, fineract_command::CommandError> {
        let column = DatatableColumn::new(
            DatatableColumnId::new(1),
            request.tenant_id,
            request.datatable_id,
            request.column_name,
            request.display_name,
            request.column_type,
            request.column_length,
            request.column_precision,
            request.column_scale,
            request.is_mandatory,
            request.is_unique,
            request.is_foreign_key,
            request.foreign_key_table,
            request.foreign_key_column,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_column(context, column)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateDatatableColumnHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> UpdateDatatableColumnHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<UpdateDatatableColumnCommand>
    for UpdateDatatableColumnHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateDatatableColumn";

    async fn handle(
        &self,
        request: UpdateDatatableColumnCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableColumnId, fineract_command::CommandError> {
        let column = DatatableColumn::new(
            request.id,
            request.tenant_id,
            request.datatable_id,
            request.column_name,
            request.display_name,
            request.column_type,
            request.column_length,
            request.column_precision,
            request.column_scale,
            request.is_mandatory,
            request.is_unique,
            request.is_foreign_key,
            request.foreign_key_table,
            request.foreign_key_column,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .update_column(context, column)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteDatatableColumnHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> DeleteDatatableColumnHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<DeleteDatatableColumnCommand>
    for DeleteDatatableColumnHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteDatatableColumn";

    async fn handle(
        &self,
        request: DeleteDatatableColumnCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableColumnId, fineract_command::CommandError> {
        self.service
            .delete_column(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct CreateDatatableViewHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> CreateDatatableViewHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<CreateDatatableViewCommand>
    for CreateDatatableViewHandler<S>
{
    const COMMAND_TYPE: &'static str = "CreateDatatableView";

    async fn handle(
        &self,
        request: CreateDatatableViewCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableViewId, fineract_command::CommandError> {
        let view = DatatableView::new(
            DatatableViewId::new(1),
            request.tenant_id,
            request.datatable_id,
            request.name,
            request.description,
            request.columns,
            request.filter_criteria,
            request.sort_criteria,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_view(context, view)
            .await
            .map(|v| v.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateDatatableViewHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> UpdateDatatableViewHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<UpdateDatatableViewCommand>
    for UpdateDatatableViewHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateDatatableView";

    async fn handle(
        &self,
        request: UpdateDatatableViewCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableViewId, fineract_command::CommandError> {
        let view = DatatableView::new(
            request.id,
            request.tenant_id,
            request.datatable_id,
            request.name,
            request.description,
            request.columns,
            request.filter_criteria,
            request.sort_criteria,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .update_view(context, view)
            .await
            .map(|v| v.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteDatatableViewHandler<S: DatatableService> {
    service: Arc<S>,
}

impl<S: DatatableService> DeleteDatatableViewHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DatatableService> CommandHandler<DeleteDatatableViewCommand>
    for DeleteDatatableViewHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteDatatableView";

    async fn handle(
        &self,
        request: DeleteDatatableViewCommand,
        context: &AuthorizedContext,
    ) -> Result<DatatableViewId, fineract_command::CommandError> {
        self.service
            .delete_view(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: DatatableService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateDatatableHandler::new(service.clone()));
    dispatcher.register_handler(UpdateDatatableHandler::new(service.clone()));
    dispatcher.register_handler(DeleteDatatableHandler::new(service.clone()));
    dispatcher.register_handler(CreateDatatableColumnHandler::new(service.clone()));
    dispatcher.register_handler(UpdateDatatableColumnHandler::new(service.clone()));
    dispatcher.register_handler(DeleteDatatableColumnHandler::new(service.clone()));
    dispatcher.register_handler(CreateDatatableViewHandler::new(service.clone()));
    dispatcher.register_handler(UpdateDatatableViewHandler::new(service.clone()));
    dispatcher.register_handler(DeleteDatatableViewHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateDatatableCommand::COMMAND_TYPE, "CreateDatatable");
        assert_eq!(UpdateDatatableCommand::COMMAND_TYPE, "UpdateDatatable");
        assert_eq!(DeleteDatatableCommand::COMMAND_TYPE, "DeleteDatatable");
        assert_eq!(
            CreateDatatableColumnCommand::COMMAND_TYPE,
            "CreateDatatableColumn"
        );
        assert_eq!(
            UpdateDatatableColumnCommand::COMMAND_TYPE,
            "UpdateDatatableColumn"
        );
        assert_eq!(
            DeleteDatatableColumnCommand::COMMAND_TYPE,
            "DeleteDatatableColumn"
        );
        assert_eq!(
            CreateDatatableViewCommand::COMMAND_TYPE,
            "CreateDatatableView"
        );
        assert_eq!(
            UpdateDatatableViewCommand::COMMAND_TYPE,
            "UpdateDatatableView"
        );
        assert_eq!(
            DeleteDatatableViewCommand::COMMAND_TYPE,
            "DeleteDatatableView"
        );
    }
}
