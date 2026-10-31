//! Command handlers for rate operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Rate;
use crate::domain::RatePeriod;
use crate::service::RateService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::RateId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRateCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub period: String,
    pub annual_percentage_rate: String,
    pub active: bool,
}

impl CommandRequest for CreateRateCommand {
    type Response = RateId;
    const COMMAND_TYPE: &'static str = "CreateRate";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRateCommand {
    pub id: RateId,
    pub tenant_id: TenantId,
    pub name: String,
    pub period: String,
    pub annual_percentage_rate: String,
    pub active: bool,
}

impl CommandRequest for UpdateRateCommand {
    type Response = RateId;
    const COMMAND_TYPE: &'static str = "UpdateRate";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRateCommand {
    pub id: RateId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteRateCommand {
    type Response = RateId;
    const COMMAND_TYPE: &'static str = "DeleteRate";
}

pub struct CreateRateHandler<S: RateService> {
    service: Arc<S>,
}

impl<S: RateService> CreateRateHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: RateService> CommandHandler<CreateRateCommand> for CreateRateHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateRate";

    async fn handle(
        &self,
        request: CreateRateCommand,
        context: &AuthorizedContext,
    ) -> Result<RateId, fineract_command::CommandError> {
        let period = match request.period.as_str() {
            "Daily" => RatePeriod::Daily,
            "Weekly" => RatePeriod::Weekly,
            "Monthly" => RatePeriod::Monthly,
            "Yearly" => RatePeriod::Yearly,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid period".to_string(),
                ))
            }
        };
        let apr = rust_decimal::Decimal::from_str_exact(&request.annual_percentage_rate)
            .map_err(|_| fineract_command::CommandError::HandlerError("invalid apr".to_string()))?;
        let rate = Rate::new(RateId::new(0), request.tenant_id, request.name, period, apr)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_rate(context, rate)
            .await
            .map(|r| r.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateRateHandler<S: RateService> {
    service: Arc<S>,
}

impl<S: RateService> UpdateRateHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: RateService> CommandHandler<UpdateRateCommand> for UpdateRateHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateRate";

    async fn handle(
        &self,
        request: UpdateRateCommand,
        context: &AuthorizedContext,
    ) -> Result<RateId, fineract_command::CommandError> {
        let period = match request.period.as_str() {
            "Daily" => RatePeriod::Daily,
            "Weekly" => RatePeriod::Weekly,
            "Monthly" => RatePeriod::Monthly,
            "Yearly" => RatePeriod::Yearly,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid period".to_string(),
                ))
            }
        };
        let apr = rust_decimal::Decimal::from_str_exact(&request.annual_percentage_rate)
            .map_err(|_| fineract_command::CommandError::HandlerError("invalid apr".to_string()))?;
        let rate = Rate::new(request.id, request.tenant_id, request.name, period, apr)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .update_rate(context, rate)
            .await
            .map(|r| r.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteRateHandler<S: RateService> {
    service: Arc<S>,
}

impl<S: RateService> DeleteRateHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: RateService> CommandHandler<DeleteRateCommand> for DeleteRateHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteRate";

    async fn handle(
        &self,
        request: DeleteRateCommand,
        context: &AuthorizedContext,
    ) -> Result<RateId, fineract_command::CommandError> {
        self.service
            .delete_rate(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: RateService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateRateHandler::new(service.clone()));
    dispatcher.register_handler(UpdateRateHandler::new(service.clone()));
    dispatcher.register_handler(DeleteRateHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateRateCommand::COMMAND_TYPE, "CreateRate");
        assert_eq!(UpdateRateCommand::COMMAND_TYPE, "UpdateRate");
        assert_eq!(DeleteRateCommand::COMMAND_TYPE, "DeleteRate");
    }
}
