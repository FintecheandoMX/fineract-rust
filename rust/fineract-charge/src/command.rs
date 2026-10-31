//! Command handlers for charge operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Charge;
use crate::domain::ChargeType;
use crate::service::ChargeService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::ChargeId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChargeCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub charge_type: String,
    pub value: String,
    pub active: bool,
}

impl CommandRequest for CreateChargeCommand {
    type Response = ChargeId;
    const COMMAND_TYPE: &'static str = "CreateCharge";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChargeCommand {
    pub id: ChargeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub charge_type: String,
    pub value: String,
    pub active: bool,
}

impl CommandRequest for UpdateChargeCommand {
    type Response = ChargeId;
    const COMMAND_TYPE: &'static str = "UpdateCharge";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteChargeCommand {
    pub id: ChargeId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteChargeCommand {
    type Response = ChargeId;
    const COMMAND_TYPE: &'static str = "DeleteCharge";
}

pub struct CreateChargeHandler<S: ChargeService> {
    service: Arc<S>,
}

impl<S: ChargeService> CreateChargeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ChargeService> CommandHandler<CreateChargeCommand> for CreateChargeHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateCharge";

    async fn handle(
        &self,
        request: CreateChargeCommand,
        context: &AuthorizedContext,
    ) -> Result<ChargeId, fineract_command::CommandError> {
        let charge_type = match request.charge_type.as_str() {
            "Fixed" => ChargeType::Fixed,
            "Percentage" => ChargeType::Percentage,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid charge type".to_string(),
                ))
            }
        };
        let value = rust_decimal::Decimal::from_str_exact(&request.value).map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid value".to_string())
        })?;
        let charge = Charge::new(
            ChargeId::new(0),
            request.tenant_id,
            request.name,
            charge_type,
            value,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_charge(context, charge)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateChargeHandler<S: ChargeService> {
    service: Arc<S>,
}

impl<S: ChargeService> UpdateChargeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ChargeService> CommandHandler<UpdateChargeCommand> for UpdateChargeHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateCharge";

    async fn handle(
        &self,
        request: UpdateChargeCommand,
        context: &AuthorizedContext,
    ) -> Result<ChargeId, fineract_command::CommandError> {
        let charge_type = match request.charge_type.as_str() {
            "Fixed" => ChargeType::Fixed,
            "Percentage" => ChargeType::Percentage,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid charge type".to_string(),
                ))
            }
        };
        let value = rust_decimal::Decimal::from_str_exact(&request.value).map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid value".to_string())
        })?;
        let charge = Charge::new(
            request.id,
            request.tenant_id,
            request.name,
            charge_type,
            value,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .update_charge(context, charge)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteChargeHandler<S: ChargeService> {
    service: Arc<S>,
}

impl<S: ChargeService> DeleteChargeHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ChargeService> CommandHandler<DeleteChargeCommand> for DeleteChargeHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteCharge";

    async fn handle(
        &self,
        request: DeleteChargeCommand,
        context: &AuthorizedContext,
    ) -> Result<ChargeId, fineract_command::CommandError> {
        self.service
            .delete_charge(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: ChargeService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateChargeHandler::new(service.clone()));
    dispatcher.register_handler(UpdateChargeHandler::new(service.clone()));
    dispatcher.register_handler(DeleteChargeHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateChargeCommand::COMMAND_TYPE, "CreateCharge");
        assert_eq!(UpdateChargeCommand::COMMAND_TYPE, "UpdateCharge");
        assert_eq!(DeleteChargeCommand::COMMAND_TYPE, "DeleteCharge");
        assert_ne!(
            CreateChargeCommand::COMMAND_TYPE,
            UpdateChargeCommand::COMMAND_TYPE
        );
    }
}
