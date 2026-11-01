//! Command handlers for code and code value operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Code;
use crate::domain::CodeValue;
use crate::service::CodeService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::CodeId;
use fineract_core::ids::CodeValueId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCodeCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

impl CommandRequest for CreateCodeCommand {
    type Response = CodeId;
    const COMMAND_TYPE: &'static str = "CreateCode";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCodeCommand {
    pub id: CodeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

impl CommandRequest for UpdateCodeCommand {
    type Response = CodeId;
    const COMMAND_TYPE: &'static str = "UpdateCode";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCodeCommand {
    pub id: CodeId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteCodeCommand {
    type Response = CodeId;
    const COMMAND_TYPE: &'static str = "DeleteCode";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCodeValueCommand {
    pub code_id: CodeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub position: i32,
    pub is_active: bool,
}

impl CommandRequest for CreateCodeValueCommand {
    type Response = CodeValueId;
    const COMMAND_TYPE: &'static str = "CreateCodeValue";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCodeValueCommand {
    pub id: CodeValueId,
    pub code_id: CodeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub position: i32,
    pub is_active: bool,
}

impl CommandRequest for UpdateCodeValueCommand {
    type Response = CodeValueId;
    const COMMAND_TYPE: &'static str = "UpdateCodeValue";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCodeValueCommand {
    pub id: CodeValueId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteCodeValueCommand {
    type Response = CodeValueId;
    const COMMAND_TYPE: &'static str = "DeleteCodeValue";
}

pub struct CreateCodeHandler<S: CodeService> {
    service: Arc<S>,
}

impl<S: CodeService> CreateCodeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CodeService> CommandHandler<CreateCodeCommand> for CreateCodeHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateCode";

    async fn handle(
        &self,
        request: CreateCodeCommand,
        context: &AuthorizedContext,
    ) -> Result<CodeId, fineract_command::CommandError> {
        let code = Code::new(
            CodeId::new(1),
            request.tenant_id,
            request.name,
            request.description,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut code = code;
        code.is_active = request.is_active;

        self.service
            .create_code(context, code)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateCodeHandler<S: CodeService> {
    service: Arc<S>,
}

impl<S: CodeService> UpdateCodeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CodeService> CommandHandler<UpdateCodeCommand> for UpdateCodeHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateCode";

    async fn handle(
        &self,
        request: UpdateCodeCommand,
        context: &AuthorizedContext,
    ) -> Result<CodeId, fineract_command::CommandError> {
        let code = Code::new(
            request.id,
            request.tenant_id,
            request.name,
            request.description,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut code = code;
        code.is_active = request.is_active;

        self.service
            .update_code(context, code)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteCodeHandler<S: CodeService> {
    service: Arc<S>,
}

impl<S: CodeService> DeleteCodeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CodeService> CommandHandler<DeleteCodeCommand> for DeleteCodeHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteCode";

    async fn handle(
        &self,
        request: DeleteCodeCommand,
        context: &AuthorizedContext,
    ) -> Result<CodeId, fineract_command::CommandError> {
        self.service
            .delete_code(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct CreateCodeValueHandler<S: CodeService> {
    service: Arc<S>,
}

impl<S: CodeService> CreateCodeValueHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CodeService> CommandHandler<CreateCodeValueCommand> for CreateCodeValueHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateCodeValue";

    async fn handle(
        &self,
        request: CreateCodeValueCommand,
        context: &AuthorizedContext,
    ) -> Result<CodeValueId, fineract_command::CommandError> {
        let cv = CodeValue::new(
            CodeValueId::new(1),
            request.code_id,
            request.tenant_id,
            request.name,
            request.description,
            request.position,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut cv = cv;
        cv.is_active = request.is_active;

        self.service
            .create_code_value(context, cv)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateCodeValueHandler<S: CodeService> {
    service: Arc<S>,
}

impl<S: CodeService> UpdateCodeValueHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CodeService> CommandHandler<UpdateCodeValueCommand> for UpdateCodeValueHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateCodeValue";

    async fn handle(
        &self,
        request: UpdateCodeValueCommand,
        context: &AuthorizedContext,
    ) -> Result<CodeValueId, fineract_command::CommandError> {
        let cv = CodeValue::new(
            request.id,
            request.code_id,
            request.tenant_id,
            request.name,
            request.description,
            request.position,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut cv = cv;
        cv.is_active = request.is_active;

        self.service
            .update_code_value(context, cv)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteCodeValueHandler<S: CodeService> {
    service: Arc<S>,
}

impl<S: CodeService> DeleteCodeValueHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CodeService> CommandHandler<DeleteCodeValueCommand> for DeleteCodeValueHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteCodeValue";

    async fn handle(
        &self,
        request: DeleteCodeValueCommand,
        context: &AuthorizedContext,
    ) -> Result<CodeValueId, fineract_command::CommandError> {
        self.service
            .delete_code_value(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: CodeService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateCodeHandler::new(service.clone()));
    dispatcher.register_handler(UpdateCodeHandler::new(service.clone()));
    dispatcher.register_handler(DeleteCodeHandler::new(service.clone()));
    dispatcher.register_handler(CreateCodeValueHandler::new(service.clone()));
    dispatcher.register_handler(UpdateCodeValueHandler::new(service.clone()));
    dispatcher.register_handler(DeleteCodeValueHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateCodeCommand::COMMAND_TYPE, "CreateCode");
        assert_eq!(UpdateCodeCommand::COMMAND_TYPE, "UpdateCode");
        assert_eq!(DeleteCodeCommand::COMMAND_TYPE, "DeleteCode");
        assert_eq!(CreateCodeValueCommand::COMMAND_TYPE, "CreateCodeValue");
        assert_eq!(UpdateCodeValueCommand::COMMAND_TYPE, "UpdateCodeValue");
        assert_eq!(DeleteCodeValueCommand::COMMAND_TYPE, "DeleteCodeValue");
    }
}
