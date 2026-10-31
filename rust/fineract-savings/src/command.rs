//! Command handlers for savings account operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::SavingsAccount;
use crate::domain::SavingsAccountStatus;
use crate::domain::SavingsAccountType;
use crate::service::SavingsAccountService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::SavingsAccountId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavingsAccountCommand {
    pub tenant_id: TenantId,
    pub account_no: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

impl CommandRequest for CreateSavingsAccountCommand {
    type Response = SavingsAccountId;
    const COMMAND_TYPE: &'static str = "CreateSavingsAccount";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavingsAccountCommand {
    pub id: SavingsAccountId,
    pub tenant_id: TenantId,
    pub account_no: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

impl CommandRequest for UpdateSavingsAccountCommand {
    type Response = SavingsAccountId;
    const COMMAND_TYPE: &'static str = "UpdateSavingsAccount";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSavingsAccountCommand {
    pub id: SavingsAccountId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteSavingsAccountCommand {
    type Response = SavingsAccountId;
    const COMMAND_TYPE: &'static str = "DeleteSavingsAccount";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateSavingsAccountCommand {
    pub id: SavingsAccountId,
    pub tenant_id: TenantId,
}

impl CommandRequest for ActivateSavingsAccountCommand {
    type Response = SavingsAccountId;
    const COMMAND_TYPE: &'static str = "ActivateSavingsAccount";
}

pub struct CreateSavingsAccountHandler<S: SavingsAccountService> {
    service: Arc<S>,
}

impl<S: SavingsAccountService> CreateSavingsAccountHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: SavingsAccountService> CommandHandler<CreateSavingsAccountCommand>
    for CreateSavingsAccountHandler<S>
{
    const COMMAND_TYPE: &'static str = "CreateSavingsAccount";

    async fn handle(
        &self,
        request: CreateSavingsAccountCommand,
        context: &AuthorizedContext,
    ) -> Result<SavingsAccountId, fineract_command::CommandError> {
        let account_type = match request.account_type.as_str() {
            "Demand" => SavingsAccountType::Demand,
            "Savings" => SavingsAccountType::Savings,
            "FixedDeposit" => SavingsAccountType::FixedDeposit,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid account type".to_string(),
                ))
            }
        };
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let account = SavingsAccount::new(
            SavingsAccountId::new(1),
            request.tenant_id,
            request.account_no,
            account_type,
            currency,
            chrono::Utc::now(),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_account(context, account)
            .await
            .map(|a| a.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateSavingsAccountHandler<S: SavingsAccountService> {
    service: Arc<S>,
}

impl<S: SavingsAccountService> UpdateSavingsAccountHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: SavingsAccountService> CommandHandler<UpdateSavingsAccountCommand>
    for UpdateSavingsAccountHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateSavingsAccount";

    async fn handle(
        &self,
        request: UpdateSavingsAccountCommand,
        context: &AuthorizedContext,
    ) -> Result<SavingsAccountId, fineract_command::CommandError> {
        let account_type = match request.account_type.as_str() {
            "Demand" => SavingsAccountType::Demand,
            "Savings" => SavingsAccountType::Savings,
            "FixedDeposit" => SavingsAccountType::FixedDeposit,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid account type".to_string(),
                ))
            }
        };
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let mut account = SavingsAccount::new(
            request.id,
            request.tenant_id,
            request.account_no,
            account_type,
            currency,
            chrono::Utc::now(),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        account.active = request.active;
        self.service
            .update_account(context, account)
            .await
            .map(|a| a.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteSavingsAccountHandler<S: SavingsAccountService> {
    service: Arc<S>,
}

impl<S: SavingsAccountService> DeleteSavingsAccountHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: SavingsAccountService> CommandHandler<DeleteSavingsAccountCommand>
    for DeleteSavingsAccountHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteSavingsAccount";

    async fn handle(
        &self,
        request: DeleteSavingsAccountCommand,
        context: &AuthorizedContext,
    ) -> Result<SavingsAccountId, fineract_command::CommandError> {
        self.service
            .delete_account(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct ActivateSavingsAccountHandler<S: SavingsAccountService> {
    service: Arc<S>,
}

impl<S: SavingsAccountService> ActivateSavingsAccountHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: SavingsAccountService> CommandHandler<ActivateSavingsAccountCommand>
    for ActivateSavingsAccountHandler<S>
{
    const COMMAND_TYPE: &'static str = "ActivateSavingsAccount";

    async fn handle(
        &self,
        request: ActivateSavingsAccountCommand,
        context: &AuthorizedContext,
    ) -> Result<SavingsAccountId, fineract_command::CommandError> {
        self.service
            .activate_account(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: SavingsAccountService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateSavingsAccountHandler::new(service.clone()));
    dispatcher.register_handler(UpdateSavingsAccountHandler::new(service.clone()));
    dispatcher.register_handler(DeleteSavingsAccountHandler::new(service.clone()));
    dispatcher.register_handler(ActivateSavingsAccountHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(
            CreateSavingsAccountCommand::COMMAND_TYPE,
            "CreateSavingsAccount"
        );
        assert_eq!(
            UpdateSavingsAccountCommand::COMMAND_TYPE,
            "UpdateSavingsAccount"
        );
        assert_eq!(
            DeleteSavingsAccountCommand::COMMAND_TYPE,
            "DeleteSavingsAccount"
        );
        assert_eq!(
            ActivateSavingsAccountCommand::COMMAND_TYPE,
            "ActivateSavingsAccount"
        );
        assert_ne!(
            CreateSavingsAccountCommand::COMMAND_TYPE,
            UpdateSavingsAccountCommand::COMMAND_TYPE
        );
    }
}
