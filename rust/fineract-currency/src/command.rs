//! Command handlers for currency operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Currency;
use crate::service::CurrencyService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::CurrencyId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCurrencyCommand {
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub display_symbol: String,
    pub display_label: String,
    pub decimal_places: i32,
    pub is_active: bool,
}

impl CommandRequest for CreateCurrencyCommand {
    type Response = CurrencyId;
    const COMMAND_TYPE: &'static str = "CreateCurrency";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCurrencyCommand {
    pub id: CurrencyId,
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub display_symbol: String,
    pub display_label: String,
    pub decimal_places: i32,
    pub is_active: bool,
}

impl CommandRequest for UpdateCurrencyCommand {
    type Response = CurrencyId;
    const COMMAND_TYPE: &'static str = "UpdateCurrency";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCurrencyCommand {
    pub id: CurrencyId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteCurrencyCommand {
    type Response = CurrencyId;
    const COMMAND_TYPE: &'static str = "DeleteCurrency";
}

pub struct CreateCurrencyHandler<S: CurrencyService> {
    service: Arc<S>,
}

impl<S: CurrencyService> CreateCurrencyHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CurrencyService> CommandHandler<CreateCurrencyCommand> for CreateCurrencyHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateCurrency";

    async fn handle(
        &self,
        request: CreateCurrencyCommand,
        context: &AuthorizedContext,
    ) -> Result<CurrencyId, fineract_command::CommandError> {
        let currency = Currency::new(
            CurrencyId::new(1),
            request.tenant_id,
            request.code,
            request.name,
            request.display_symbol,
            request.display_label,
            request.decimal_places,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut currency = currency;
        currency.is_active = request.is_active;

        self.service
            .create_currency(context, currency)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateCurrencyHandler<S: CurrencyService> {
    service: Arc<S>,
}

impl<S: CurrencyService> UpdateCurrencyHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CurrencyService> CommandHandler<UpdateCurrencyCommand> for UpdateCurrencyHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateCurrency";

    async fn handle(
        &self,
        request: UpdateCurrencyCommand,
        context: &AuthorizedContext,
    ) -> Result<CurrencyId, fineract_command::CommandError> {
        let currency = Currency::new(
            request.id,
            request.tenant_id,
            request.code,
            request.name,
            request.display_symbol,
            request.display_label,
            request.decimal_places,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut currency = currency;
        currency.is_active = request.is_active;

        self.service
            .update_currency(context, currency)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteCurrencyHandler<S: CurrencyService> {
    service: Arc<S>,
}

impl<S: CurrencyService> DeleteCurrencyHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CurrencyService> CommandHandler<DeleteCurrencyCommand> for DeleteCurrencyHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteCurrency";

    async fn handle(
        &self,
        request: DeleteCurrencyCommand,
        context: &AuthorizedContext,
    ) -> Result<CurrencyId, fineract_command::CommandError> {
        self.service
            .delete_currency(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: CurrencyService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateCurrencyHandler::new(service.clone()));
    dispatcher.register_handler(UpdateCurrencyHandler::new(service.clone()));
    dispatcher.register_handler(DeleteCurrencyHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateCurrencyCommand::COMMAND_TYPE, "CreateCurrency");
        assert_eq!(UpdateCurrencyCommand::COMMAND_TYPE, "UpdateCurrency");
        assert_eq!(DeleteCurrencyCommand::COMMAND_TYPE, "DeleteCurrency");
    }
}
