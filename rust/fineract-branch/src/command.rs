//! Command handlers for branch operations.
//!
//! All write operations go through the command pipeline (Phase 3), which
//! enforces idempotency, maker-checker, and audit logging. Each command
//! handler calls the domain service, which enforces authorization.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Office;

use crate::service::BranchService;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_command::CommandHandler;
use fineract_command::CommandProcessingResult;
use fineract_command::CommandRequest;
use fineract_core::ids::OfficeId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;
use fineract_security::AuthorizedContext;

/// Create office command payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOfficeCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub external_id: Option<String>,
    pub opening_date: Option<chrono::NaiveDate>,
    pub timezone: Option<String>,
    pub country: Option<String>,
    pub currency: String,
    pub parent_office_id: Option<OfficeId>,
    pub active: bool,
}

impl CommandRequest for CreateOfficeCommand {
    type Response = OfficeId;
    const COMMAND_TYPE: &'static str = "CreateOffice";
}

/// Update office command payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOfficeCommand {
    pub id: OfficeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub external_id: Option<String>,
    pub opening_date: Option<chrono::NaiveDate>,
    pub timezone: Option<String>,
    pub country: Option<String>,
    pub currency: String,
    pub parent_office_id: Option<OfficeId>,
    pub active: bool,
}

impl CommandRequest for UpdateOfficeCommand {
    type Response = OfficeId;
    const COMMAND_TYPE: &'static str = "UpdateOffice";
}

/// Delete office command payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOfficeCommand {
    pub id: OfficeId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteOfficeCommand {
    type Response = OfficeId;
    const COMMAND_TYPE: &'static str = "DeleteOffice";
}

/// Command handler for CreateOffice.
pub struct CreateOfficeHandler<S: BranchService> {
    service: Arc<S>,
}

impl<S: BranchService> CreateOfficeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: BranchService> CommandHandler<CreateOfficeCommand> for CreateOfficeHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateOffice";

    async fn handle(
        &self,
        request: CreateOfficeCommand,
        context: &AuthorizedContext,
    ) -> Result<OfficeId, fineract_command::CommandError> {
        let currency = Currency::from_code(&request.currency).map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid currency code".to_string())
        })?;

        // TODO: Replace placeholder ID with server-side ID generation (sequence/UUID).
        let office = Office::new(
            OfficeId::new(1),
            request.tenant_id,
            request.name.clone(),
            currency,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let office = self
            .service
            .create_office(context, office)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        Ok(office.id)
    }
}

/// Command handler for UpdateOffice.
pub struct UpdateOfficeHandler<S: BranchService> {
    service: Arc<S>,
}

impl<S: BranchService> UpdateOfficeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: BranchService> CommandHandler<UpdateOfficeCommand> for UpdateOfficeHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateOffice";

    async fn handle(
        &self,
        request: UpdateOfficeCommand,
        context: &AuthorizedContext,
    ) -> Result<OfficeId, fineract_command::CommandError> {
        let currency = Currency::from_code(&request.currency).map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid currency code".to_string())
        })?;

        let office = Office::new(
            request.id,
            request.tenant_id,
            request.name.clone(),
            currency,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .update_office(context, office)
            .await
            .map(|o| o.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

/// Command handler for DeleteOffice.
pub struct DeleteOfficeHandler<S: BranchService> {
    service: Arc<S>,
}

impl<S: BranchService> DeleteOfficeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: BranchService> CommandHandler<DeleteOfficeCommand> for DeleteOfficeHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteOffice";

    async fn handle(
        &self,
        request: DeleteOfficeCommand,
        context: &AuthorizedContext,
    ) -> Result<OfficeId, fineract_command::CommandError> {
        self.service
            .delete_office(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

/// Register all branch command handlers with a dispatcher.
pub fn register_handlers<S: BranchService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateOfficeHandler::new(service.clone()));
    dispatcher.register_handler(UpdateOfficeHandler::new(service.clone()));
    dispatcher.register_handler(DeleteOfficeHandler::new(service));
}

/// Execute a CreateOffice command through the dispatcher.
pub async fn execute_create_office<S: BranchService + 'static>(
    dispatcher: &fineract_command::SynchronousCommandDispatcher,
    command: CreateOfficeCommand,
    context: &AuthorizedContext,
) -> Result<CommandProcessingResult<CreateOfficeCommand>, fineract_command::CommandError> {
    let cmd = Command::new(
        None,
        command.tenant_id,
        Some(context.principal.username.clone()),
        command,
    );
    dispatcher.dispatch(cmd, context).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateOfficeCommand::COMMAND_TYPE, "CreateOffice");
        assert_eq!(UpdateOfficeCommand::COMMAND_TYPE, "UpdateOffice");
        assert_eq!(DeleteOfficeCommand::COMMAND_TYPE, "DeleteOffice");
        assert_ne!(
            CreateOfficeCommand::COMMAND_TYPE,
            UpdateOfficeCommand::COMMAND_TYPE
        );
    }
}
