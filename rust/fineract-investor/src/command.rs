//! Command handlers for investor operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Investor;
use crate::service::InvestorService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::InvestorId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvestorCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub external_id: Option<String>,
    pub active: bool,
}

impl CommandRequest for CreateInvestorCommand {
    type Response = InvestorId;
    const COMMAND_TYPE: &'static str = "CreateInvestor";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInvestorCommand {
    pub id: InvestorId,
    pub tenant_id: TenantId,
    pub name: String,
    pub external_id: Option<String>,
    pub active: bool,
}

impl CommandRequest for UpdateInvestorCommand {
    type Response = InvestorId;
    const COMMAND_TYPE: &'static str = "UpdateInvestor";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteInvestorCommand {
    pub id: InvestorId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteInvestorCommand {
    type Response = InvestorId;
    const COMMAND_TYPE: &'static str = "DeleteInvestor";
}

pub struct CreateInvestorHandler<S: InvestorService> {
    service: Arc<S>,
}

impl<S: InvestorService> CreateInvestorHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: InvestorService> CommandHandler<CreateInvestorCommand> for CreateInvestorHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateInvestor";

    async fn handle(
        &self,
        request: CreateInvestorCommand,
        context: &AuthorizedContext,
    ) -> Result<InvestorId, fineract_command::CommandError> {
        let investor = Investor::new(
            InvestorId::new(1),
            request.tenant_id,
            request.name,
            request.external_id,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_investor(context, investor)
            .await
            .map(|i| i.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateInvestorHandler<S: InvestorService> {
    service: Arc<S>,
}

impl<S: InvestorService> UpdateInvestorHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: InvestorService> CommandHandler<UpdateInvestorCommand> for UpdateInvestorHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateInvestor";

    async fn handle(
        &self,
        request: UpdateInvestorCommand,
        context: &AuthorizedContext,
    ) -> Result<InvestorId, fineract_command::CommandError> {
        let investor = Investor::new(
            request.id,
            request.tenant_id,
            request.name,
            request.external_id,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .update_investor(context, investor)
            .await
            .map(|i| i.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteInvestorHandler<S: InvestorService> {
    service: Arc<S>,
}

impl<S: InvestorService> DeleteInvestorHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: InvestorService> CommandHandler<DeleteInvestorCommand> for DeleteInvestorHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteInvestor";

    async fn handle(
        &self,
        request: DeleteInvestorCommand,
        context: &AuthorizedContext,
    ) -> Result<InvestorId, fineract_command::CommandError> {
        self.service
            .delete_investor(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: InvestorService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateInvestorHandler::new(service.clone()));
    dispatcher.register_handler(UpdateInvestorHandler::new(service.clone()));
    dispatcher.register_handler(DeleteInvestorHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateInvestorCommand::COMMAND_TYPE, "CreateInvestor");
        assert_eq!(UpdateInvestorCommand::COMMAND_TYPE, "UpdateInvestor");
        assert_eq!(DeleteInvestorCommand::COMMAND_TYPE, "DeleteInvestor");
    }
}
