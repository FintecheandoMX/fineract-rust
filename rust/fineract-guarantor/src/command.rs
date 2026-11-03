//! Command handlers for guarantor operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Guarantor;
use crate::service::GuarantorService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::GuarantorId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGuarantorCommand {
    pub tenant_id: TenantId,
    pub loan_id: LoanId,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub identification_number: Option<String>,
    pub net_worth: Option<String>,
}

impl CommandRequest for CreateGuarantorCommand {
    type Response = GuarantorId;
    const COMMAND_TYPE: &'static str = "CreateGuarantor";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGuarantorCommand {
    pub id: GuarantorId,
    pub tenant_id: TenantId,
    pub loan_id: LoanId,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub identification_number: Option<String>,
    pub net_worth: Option<String>,
    pub active: bool,
}

impl CommandRequest for UpdateGuarantorCommand {
    type Response = GuarantorId;
    const COMMAND_TYPE: &'static str = "UpdateGuarantor";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteGuarantorCommand {
    pub id: GuarantorId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteGuarantorCommand {
    type Response = GuarantorId;
    const COMMAND_TYPE: &'static str = "DeleteGuarantor";
}

pub struct CreateGuarantorHandler<S: GuarantorService> {
    service: Arc<S>,
}

impl<S: GuarantorService> CreateGuarantorHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: GuarantorService> CommandHandler<CreateGuarantorCommand> for CreateGuarantorHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateGuarantor";

    async fn handle(
        &self,
        request: CreateGuarantorCommand,
        context: &AuthorizedContext,
    ) -> Result<GuarantorId, fineract_command::CommandError> {
        let guarantor = Guarantor::new(
            GuarantorId::new(1),
            request.tenant_id,
            request.loan_id,
            request.name,
            request.email,
            request.phone,
            request.address,
            request.identification_number,
            request.net_worth,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_guarantor(context, guarantor)
            .await
            .map(|g| g.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateGuarantorHandler<S: GuarantorService> {
    service: Arc<S>,
}

impl<S: GuarantorService> UpdateGuarantorHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: GuarantorService> CommandHandler<UpdateGuarantorCommand> for UpdateGuarantorHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateGuarantor";

    async fn handle(
        &self,
        request: UpdateGuarantorCommand,
        context: &AuthorizedContext,
    ) -> Result<GuarantorId, fineract_command::CommandError> {
        let guarantor = Guarantor::new(
            request.id,
            request.tenant_id,
            request.loan_id,
            request.name,
            request.email,
            request.phone,
            request.address,
            request.identification_number,
            request.net_worth,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut guarantor = guarantor;
        guarantor.active = request.active;

        self.service
            .update_guarantor(context, guarantor)
            .await
            .map(|g| g.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteGuarantorHandler<S: GuarantorService> {
    service: Arc<S>,
}

impl<S: GuarantorService> DeleteGuarantorHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: GuarantorService> CommandHandler<DeleteGuarantorCommand> for DeleteGuarantorHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteGuarantor";

    async fn handle(
        &self,
        request: DeleteGuarantorCommand,
        context: &AuthorizedContext,
    ) -> Result<GuarantorId, fineract_command::CommandError> {
        self.service
            .delete_guarantor(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: GuarantorService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateGuarantorHandler::new(service.clone()));
    dispatcher.register_handler(UpdateGuarantorHandler::new(service.clone()));
    dispatcher.register_handler(DeleteGuarantorHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateGuarantorCommand::COMMAND_TYPE, "CreateGuarantor");
        assert_eq!(UpdateGuarantorCommand::COMMAND_TYPE, "UpdateGuarantor");
        assert_eq!(DeleteGuarantorCommand::COMMAND_TYPE, "DeleteGuarantor");
    }
}
