//! Command handlers for tax operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Tax;
use crate::domain::TaxInclusion;
use crate::service::TaxService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::TaxId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaxCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub rate: String,
    pub inclusion: String,
    pub active: bool,
}

impl CommandRequest for CreateTaxCommand {
    type Response = TaxId;
    const COMMAND_TYPE: &'static str = "CreateTax";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaxCommand {
    pub id: TaxId,
    pub tenant_id: TenantId,
    pub name: String,
    pub rate: String,
    pub inclusion: String,
    pub active: bool,
}

impl CommandRequest for UpdateTaxCommand {
    type Response = TaxId;
    const COMMAND_TYPE: &'static str = "UpdateTax";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTaxCommand {
    pub id: TaxId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteTaxCommand {
    type Response = TaxId;
    const COMMAND_TYPE: &'static str = "DeleteTax";
}

pub struct CreateTaxHandler<S: TaxService> {
    service: Arc<S>,
}

impl<S: TaxService> CreateTaxHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: TaxService> CommandHandler<CreateTaxCommand> for CreateTaxHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateTax";

    async fn handle(
        &self,
        request: CreateTaxCommand,
        context: &AuthorizedContext,
    ) -> Result<TaxId, fineract_command::CommandError> {
        let inclusion = match request.inclusion.as_str() {
            "Included" => TaxInclusion::Included,
            "Excluded" => TaxInclusion::Excluded,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid inclusion".to_string(),
                ))
            }
        };
        let rate = rust_decimal::Decimal::from_str_exact(&request.rate).map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid rate".to_string())
        })?;
        let tax = Tax::new(
            TaxId::new(1),
            request.tenant_id,
            request.name,
            rate,
            inclusion,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_tax(context, tax)
            .await
            .map(|t| t.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateTaxHandler<S: TaxService> {
    service: Arc<S>,
}

impl<S: TaxService> UpdateTaxHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: TaxService> CommandHandler<UpdateTaxCommand> for UpdateTaxHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateTax";

    async fn handle(
        &self,
        request: UpdateTaxCommand,
        context: &AuthorizedContext,
    ) -> Result<TaxId, fineract_command::CommandError> {
        let inclusion = match request.inclusion.as_str() {
            "Included" => TaxInclusion::Included,
            "Excluded" => TaxInclusion::Excluded,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid inclusion".to_string(),
                ))
            }
        };
        let rate = rust_decimal::Decimal::from_str_exact(&request.rate).map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid rate".to_string())
        })?;
        let tax = Tax::new(request.id, request.tenant_id, request.name, rate, inclusion)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .update_tax(context, tax)
            .await
            .map(|t| t.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteTaxHandler<S: TaxService> {
    service: Arc<S>,
}

impl<S: TaxService> DeleteTaxHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: TaxService> CommandHandler<DeleteTaxCommand> for DeleteTaxHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteTax";

    async fn handle(
        &self,
        request: DeleteTaxCommand,
        context: &AuthorizedContext,
    ) -> Result<TaxId, fineract_command::CommandError> {
        self.service
            .delete_tax(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: TaxService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateTaxHandler::new(service.clone()));
    dispatcher.register_handler(UpdateTaxHandler::new(service.clone()));
    dispatcher.register_handler(DeleteTaxHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateTaxCommand::COMMAND_TYPE, "CreateTax");
        assert_eq!(UpdateTaxCommand::COMMAND_TYPE, "UpdateTax");
        assert_eq!(DeleteTaxCommand::COMMAND_TYPE, "DeleteTax");
    }
}
