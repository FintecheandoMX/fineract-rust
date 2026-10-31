//! Command handlers for accounting operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Account;
use crate::domain::AccountStatus;
use crate::domain::AccountType;
use crate::service::AccountService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::AccountId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountCommand {
    pub tenant_id: TenantId,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

impl CommandRequest for CreateAccountCommand {
    type Response = AccountId;
    const COMMAND_TYPE: &'static str = "CreateAccount";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountCommand {
    pub id: AccountId,
    pub tenant_id: TenantId,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub currency: String,
    pub active: bool,
}

impl CommandRequest for UpdateAccountCommand {
    type Response = AccountId;
    const COMMAND_TYPE: &'static str = "UpdateAccount";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAccountCommand {
    pub id: AccountId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteAccountCommand {
    type Response = AccountId;
    const COMMAND_TYPE: &'static str = "DeleteAccount";
}

pub struct CreateAccountHandler<S: AccountService> {
    service: Arc<S>,
}

impl<S: AccountService> CreateAccountHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: AccountService> CommandHandler<CreateAccountCommand> for CreateAccountHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateAccount";

    async fn handle(
        &self,
        request: CreateAccountCommand,
        context: &AuthorizedContext,
    ) -> Result<AccountId, fineract_command::CommandError> {
        let account_type = match request.account_type.as_str() {
            "Asset" => AccountType::Asset,
            "Liability" => AccountType::Liability,
            "Equity" => AccountType::Equity,
            "Revenue" => AccountType::Revenue,
            "Expense" => AccountType::Expense,
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
        let account = Account::new(
            AccountId::new(1),
            request.tenant_id,
            request.account_code,
            request.account_name,
            account_type,
            currency,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_account(context, account)
            .await
            .map(|a| a.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateAccountHandler<S: AccountService> {
    service: Arc<S>,
}

impl<S: AccountService> UpdateAccountHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: AccountService> CommandHandler<UpdateAccountCommand> for UpdateAccountHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateAccount";

    async fn handle(
        &self,
        request: UpdateAccountCommand,
        context: &AuthorizedContext,
    ) -> Result<AccountId, fineract_command::CommandError> {
        let account_type = match request.account_type.as_str() {
            "Asset" => AccountType::Asset,
            "Liability" => AccountType::Liability,
            "Equity" => AccountType::Equity,
            "Revenue" => AccountType::Revenue,
            "Expense" => AccountType::Expense,
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
        let mut account = Account::new(
            request.id,
            request.tenant_id,
            request.account_code,
            request.account_name,
            account_type,
            currency,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        account.active = request.active;
        account.status = AccountStatus::Active;
        self.service
            .update_account(context, account)
            .await
            .map(|a| a.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteAccountHandler<S: AccountService> {
    service: Arc<S>,
}

impl<S: AccountService> DeleteAccountHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: AccountService> CommandHandler<DeleteAccountCommand> for DeleteAccountHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteAccount";

    async fn handle(
        &self,
        request: DeleteAccountCommand,
        context: &AuthorizedContext,
    ) -> Result<AccountId, fineract_command::CommandError> {
        self.service
            .delete_account(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: AccountService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateAccountHandler::new(service.clone()));
    dispatcher.register_handler(UpdateAccountHandler::new(service.clone()));
    dispatcher.register_handler(DeleteAccountHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateAccountCommand::COMMAND_TYPE, "CreateAccount");
        assert_eq!(UpdateAccountCommand::COMMAND_TYPE, "UpdateAccount");
        assert_eq!(DeleteAccountCommand::COMMAND_TYPE, "DeleteAccount");
        assert_ne!(
            CreateAccountCommand::COMMAND_TYPE,
            UpdateAccountCommand::COMMAND_TYPE
        );
    }
}
