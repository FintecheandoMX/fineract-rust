//! Command handlers for working capital loan operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{WorkingCapitalLoan, WorkingCapitalLoanStatus};
use crate::service::WorkingCapitalLoanService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkingCapitalLoanCommand {
    pub tenant_id: TenantId,
    pub client_id: i64,
    pub credit_limit: String,
    pub currency: String,
    pub interest_rate: String,
}

impl CommandRequest for CreateWorkingCapitalLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "CreateWorkingCapitalLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkingCapitalLoanCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub client_id: i64,
    pub credit_limit: String,
    pub currency: String,
    pub interest_rate: String,
}

impl CommandRequest for UpdateWorkingCapitalLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "UpdateWorkingCapitalLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWorkingCapitalLoanCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteWorkingCapitalLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "DeleteWorkingCapitalLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkingCapitalLoanStatusCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub status: String,
}

impl CommandRequest for UpdateWorkingCapitalLoanStatusCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "UpdateWorkingCapitalLoanStatus";
}

pub struct CreateWorkingCapitalLoanHandler<S: WorkingCapitalLoanService> {
    service: Arc<S>,
}

impl<S: WorkingCapitalLoanService> CreateWorkingCapitalLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: WorkingCapitalLoanService> CommandHandler<CreateWorkingCapitalLoanCommand>
    for CreateWorkingCapitalLoanHandler<S>
{
    const COMMAND_TYPE: &'static str = "CreateWorkingCapitalLoan";

    async fn handle(
        &self,
        request: CreateWorkingCapitalLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let credit_limit =
            rust_decimal::Decimal::from_str_exact(&request.credit_limit).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid credit limit".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let interest_rate =
            rust_decimal::Decimal::from_str_exact(&request.interest_rate).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid interest rate".to_string())
            })?;

        let loan = WorkingCapitalLoan::new(
            LoanId::new(1),
            request.tenant_id,
            fineract_core::ids::ClientId::new(request.client_id),
            credit_limit,
            currency,
            interest_rate,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_working_capital_loan(context, loan)
            .await
            .map(|l| l.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateWorkingCapitalLoanHandler<S: WorkingCapitalLoanService> {
    service: Arc<S>,
}

impl<S: WorkingCapitalLoanService> UpdateWorkingCapitalLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: WorkingCapitalLoanService> CommandHandler<UpdateWorkingCapitalLoanCommand>
    for UpdateWorkingCapitalLoanHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateWorkingCapitalLoan";

    async fn handle(
        &self,
        request: UpdateWorkingCapitalLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let credit_limit =
            rust_decimal::Decimal::from_str_exact(&request.credit_limit).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid credit limit".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let interest_rate =
            rust_decimal::Decimal::from_str_exact(&request.interest_rate).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid interest rate".to_string())
            })?;

        let loan = WorkingCapitalLoan::new(
            request.id,
            request.tenant_id,
            fineract_core::ids::ClientId::new(request.client_id),
            credit_limit,
            currency,
            interest_rate,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .update_working_capital_loan(context, loan)
            .await
            .map(|l| l.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteWorkingCapitalLoanHandler<S: WorkingCapitalLoanService> {
    service: Arc<S>,
}

impl<S: WorkingCapitalLoanService> DeleteWorkingCapitalLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: WorkingCapitalLoanService> CommandHandler<DeleteWorkingCapitalLoanCommand>
    for DeleteWorkingCapitalLoanHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteWorkingCapitalLoan";

    async fn handle(
        &self,
        request: DeleteWorkingCapitalLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        self.service
            .delete_working_capital_loan(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateWorkingCapitalLoanStatusHandler<S: WorkingCapitalLoanService> {
    service: Arc<S>,
}

impl<S: WorkingCapitalLoanService> UpdateWorkingCapitalLoanStatusHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: WorkingCapitalLoanService> CommandHandler<UpdateWorkingCapitalLoanStatusCommand>
    for UpdateWorkingCapitalLoanStatusHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateWorkingCapitalLoanStatus";

    async fn handle(
        &self,
        request: UpdateWorkingCapitalLoanStatusCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let status = match request.status.as_str() {
            "Active" => WorkingCapitalLoanStatus::Active,
            "Suspended" => WorkingCapitalLoanStatus::Suspended,
            "Closed" => WorkingCapitalLoanStatus::Closed,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid status".to_string(),
                ))
            }
        };
        self.service
            .update_status(context, request.id, status)
            .await
            .map(|()| request.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: WorkingCapitalLoanService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateWorkingCapitalLoanHandler::new(service.clone()));
    dispatcher.register_handler(UpdateWorkingCapitalLoanHandler::new(service.clone()));
    dispatcher.register_handler(DeleteWorkingCapitalLoanHandler::new(service.clone()));
    dispatcher.register_handler(UpdateWorkingCapitalLoanStatusHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(
            CreateWorkingCapitalLoanCommand::COMMAND_TYPE,
            "CreateWorkingCapitalLoan"
        );
        assert_eq!(
            UpdateWorkingCapitalLoanCommand::COMMAND_TYPE,
            "UpdateWorkingCapitalLoan"
        );
        assert_eq!(
            DeleteWorkingCapitalLoanCommand::COMMAND_TYPE,
            "DeleteWorkingCapitalLoan"
        );
        assert_eq!(
            UpdateWorkingCapitalLoanStatusCommand::COMMAND_TYPE,
            "UpdateWorkingCapitalLoanStatus"
        );
    }
}
