//! Command handlers for progressive loan operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{ProgressiveLoan, ProgressiveLoanPhase};
use crate::service::ProgressiveLoanService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProgressiveLoanCommand {
    pub tenant_id: TenantId,
    pub client_id: i64,
    pub total_amount: String,
    pub currency: String,
    pub interest_rate: String,
}

impl CommandRequest for CreateProgressiveLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "CreateProgressiveLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressiveLoanCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub client_id: i64,
    pub total_amount: String,
    pub currency: String,
    pub interest_rate: String,
}

impl CommandRequest for UpdateProgressiveLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "UpdateProgressiveLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancePhaseCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub phase: String,
}

impl CommandRequest for AdvancePhaseCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "AdvancePhase";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProgressiveLoanCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteProgressiveLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "DeleteProgressiveLoan";
}

pub struct CreateProgressiveLoanHandler<S: ProgressiveLoanService> {
    service: Arc<S>,
}

impl<S: ProgressiveLoanService> CreateProgressiveLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ProgressiveLoanService> CommandHandler<CreateProgressiveLoanCommand>
    for CreateProgressiveLoanHandler<S>
{
    const COMMAND_TYPE: &'static str = "CreateProgressiveLoan";

    async fn handle(
        &self,
        request: CreateProgressiveLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let total_amount =
            rust_decimal::Decimal::from_str_exact(&request.total_amount).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid total amount".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let interest_rate =
            rust_decimal::Decimal::from_str_exact(&request.interest_rate).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid interest rate".to_string())
            })?;

        let loan = ProgressiveLoan::new(
            LoanId::new(1),
            request.tenant_id,
            fineract_core::ids::ClientId::new(request.client_id),
            total_amount,
            currency,
            interest_rate,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_progressive_loan(context, loan)
            .await
            .map(|l| l.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateProgressiveLoanHandler<S: ProgressiveLoanService> {
    service: Arc<S>,
}

impl<S: ProgressiveLoanService> UpdateProgressiveLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ProgressiveLoanService> CommandHandler<UpdateProgressiveLoanCommand>
    for UpdateProgressiveLoanHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateProgressiveLoan";

    async fn handle(
        &self,
        request: UpdateProgressiveLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let total_amount =
            rust_decimal::Decimal::from_str_exact(&request.total_amount).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid total amount".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let interest_rate =
            rust_decimal::Decimal::from_str_exact(&request.interest_rate).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid interest rate".to_string())
            })?;

        let loan = ProgressiveLoan::new(
            request.id,
            request.tenant_id,
            fineract_core::ids::ClientId::new(request.client_id),
            total_amount,
            currency,
            interest_rate,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .update_progressive_loan(context, loan)
            .await
            .map(|l| l.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct AdvancePhaseHandler<S: ProgressiveLoanService> {
    service: Arc<S>,
}

impl<S: ProgressiveLoanService> AdvancePhaseHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ProgressiveLoanService> CommandHandler<AdvancePhaseCommand> for AdvancePhaseHandler<S> {
    const COMMAND_TYPE: &'static str = "AdvancePhase";

    async fn handle(
        &self,
        request: AdvancePhaseCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let phase = match request.phase.as_str() {
            "Application" => ProgressiveLoanPhase::Application,
            "Approved" => ProgressiveLoanPhase::Approved,
            "Disbursed" => ProgressiveLoanPhase::Disbursed,
            "Completed" => ProgressiveLoanPhase::Completed,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid phase".to_string(),
                ))
            }
        };
        self.service
            .advance_phase(context, request.id, phase)
            .await
            .map(|()| request.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteProgressiveLoanHandler<S: ProgressiveLoanService> {
    service: Arc<S>,
}

impl<S: ProgressiveLoanService> DeleteProgressiveLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ProgressiveLoanService> CommandHandler<DeleteProgressiveLoanCommand>
    for DeleteProgressiveLoanHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteProgressiveLoan";

    async fn handle(
        &self,
        request: DeleteProgressiveLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        self.service
            .delete_progressive_loan(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: ProgressiveLoanService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateProgressiveLoanHandler::new(service.clone()));
    dispatcher.register_handler(UpdateProgressiveLoanHandler::new(service.clone()));
    dispatcher.register_handler(AdvancePhaseHandler::new(service.clone()));
    dispatcher.register_handler(DeleteProgressiveLoanHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(
            CreateProgressiveLoanCommand::COMMAND_TYPE,
            "CreateProgressiveLoan"
        );
        assert_eq!(
            UpdateProgressiveLoanCommand::COMMAND_TYPE,
            "UpdateProgressiveLoan"
        );
        assert_eq!(AdvancePhaseCommand::COMMAND_TYPE, "AdvancePhase");
        assert_eq!(
            DeleteProgressiveLoanCommand::COMMAND_TYPE,
            "DeleteProgressiveLoan"
        );
    }
}
