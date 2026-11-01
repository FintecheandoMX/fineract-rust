//! Command handlers for savings product operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::SavingsProduct;
use crate::service::SavingsProductService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::SavingsProductId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavingsProductCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub currency_digits: i32,
    pub interest_rate: String,
    pub interest_compounding_period_type: i32,
    pub interest_posting_period_type: i32,
    pub interest_calculation_type: i32,
    pub interest_calculation_days_in_year_type: i32,
    pub min_required_opening_balance: String,
    pub max_balance: Option<String>,
    pub min_deposit_amount: Option<String>,
    pub max_deposit_amount: Option<String>,
    pub min_withdrawal_amount: Option<String>,
    pub max_withdrawal_amount: Option<String>,
    pub allow_overdraft: bool,
    pub overdraft_limit: Option<String>,
    pub nominal_annual_interest_rate: String,
    pub annual_interest_rate: String,
    pub min_balance_to_earn_interest: Option<String>,
    pub account_fee: Option<String>,
    pub maintenance_fee: Option<String>,
    pub withdrawal_fee: Option<String>,
    pub deposit_fee: Option<String>,
    pub inactive_account_fee: Option<String>,
    pub reject_if_insufficient_funds: bool,
    pub is_active: bool,
}

impl CommandRequest for CreateSavingsProductCommand {
    type Response = SavingsProductId;
    const COMMAND_TYPE: &'static str = "CreateSavingsProduct";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavingsProductCommand {
    pub id: SavingsProductId,
    pub tenant_id: TenantId,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub currency_digits: i32,
    pub interest_rate: String,
    pub interest_compounding_period_type: i32,
    pub interest_posting_period_type: i32,
    pub interest_calculation_type: i32,
    pub interest_calculation_days_in_year_type: i32,
    pub min_required_opening_balance: String,
    pub max_balance: Option<String>,
    pub min_deposit_amount: Option<String>,
    pub max_deposit_amount: Option<String>,
    pub min_withdrawal_amount: Option<String>,
    pub max_withdrawal_amount: Option<String>,
    pub allow_overdraft: bool,
    pub overdraft_limit: Option<String>,
    pub nominal_annual_interest_rate: String,
    pub annual_interest_rate: String,
    pub min_balance_to_earn_interest: Option<String>,
    pub account_fee: Option<String>,
    pub maintenance_fee: Option<String>,
    pub withdrawal_fee: Option<String>,
    pub deposit_fee: Option<String>,
    pub inactive_account_fee: Option<String>,
    pub reject_if_insufficient_funds: bool,
    pub is_active: bool,
}

impl CommandRequest for UpdateSavingsProductCommand {
    type Response = SavingsProductId;
    const COMMAND_TYPE: &'static str = "UpdateSavingsProduct";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSavingsProductCommand {
    pub id: SavingsProductId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteSavingsProductCommand {
    type Response = SavingsProductId;
    const COMMAND_TYPE: &'static str = "DeleteSavingsProduct";
}

pub struct CreateSavingsProductHandler<S: SavingsProductService> {
    service: Arc<S>,
}

impl<S: SavingsProductService> CreateSavingsProductHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: SavingsProductService> CommandHandler<CreateSavingsProductCommand>
    for CreateSavingsProductHandler<S>
{
    const COMMAND_TYPE: &'static str = "CreateSavingsProduct";

    async fn handle(
        &self,
        request: CreateSavingsProductCommand,
        context: &AuthorizedContext,
    ) -> Result<SavingsProductId, fineract_command::CommandError> {
        let product = SavingsProduct::new(
            SavingsProductId::new(1),
            request.tenant_id,
            request.name,
            request.short_name,
            request.description,
            request.currency_code,
            request.interest_rate,
            request.nominal_annual_interest_rate,
            request.annual_interest_rate,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut product = product;
        product.currency_digits = request.currency_digits;
        product.interest_compounding_period_type = request.interest_compounding_period_type;
        product.interest_posting_period_type = request.interest_posting_period_type;
        product.interest_calculation_type = request.interest_calculation_type;
        product.interest_calculation_days_in_year_type =
            request.interest_calculation_days_in_year_type;
        product.min_required_opening_balance = request.min_required_opening_balance;
        product.max_balance = request.max_balance;
        product.min_deposit_amount = request.min_deposit_amount;
        product.max_deposit_amount = request.max_deposit_amount;
        product.min_withdrawal_amount = request.min_withdrawal_amount;
        product.max_withdrawal_amount = request.max_withdrawal_amount;
        product.allow_overdraft = request.allow_overdraft;
        product.overdraft_limit = request.overdraft_limit;
        product.min_balance_to_earn_interest = request.min_balance_to_earn_interest;
        product.account_fee = request.account_fee;
        product.maintenance_fee = request.maintenance_fee;
        product.withdrawal_fee = request.withdrawal_fee;
        product.deposit_fee = request.deposit_fee;
        product.inactive_account_fee = request.inactive_account_fee;
        product.reject_if_insufficient_funds = request.reject_if_insufficient_funds;
        product.is_active = request.is_active;

        self.service
            .create_savings_product(context, product)
            .await
            .map(|p| p.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateSavingsProductHandler<S: SavingsProductService> {
    service: Arc<S>,
}

impl<S: SavingsProductService> UpdateSavingsProductHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: SavingsProductService> CommandHandler<UpdateSavingsProductCommand>
    for UpdateSavingsProductHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateSavingsProduct";

    async fn handle(
        &self,
        request: UpdateSavingsProductCommand,
        context: &AuthorizedContext,
    ) -> Result<SavingsProductId, fineract_command::CommandError> {
        let product = SavingsProduct::new(
            request.id,
            request.tenant_id,
            request.name,
            request.short_name,
            request.description,
            request.currency_code,
            request.interest_rate,
            request.nominal_annual_interest_rate,
            request.annual_interest_rate,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut product = product;
        product.currency_digits = request.currency_digits;
        product.interest_compounding_period_type = request.interest_compounding_period_type;
        product.interest_posting_period_type = request.interest_posting_period_type;
        product.interest_calculation_type = request.interest_calculation_type;
        product.interest_calculation_days_in_year_type =
            request.interest_calculation_days_in_year_type;
        product.min_required_opening_balance = request.min_required_opening_balance;
        product.max_balance = request.max_balance;
        product.min_deposit_amount = request.min_deposit_amount;
        product.max_deposit_amount = request.max_deposit_amount;
        product.min_withdrawal_amount = request.min_withdrawal_amount;
        product.max_withdrawal_amount = request.max_withdrawal_amount;
        product.allow_overdraft = request.allow_overdraft;
        product.overdraft_limit = request.overdraft_limit;
        product.min_balance_to_earn_interest = request.min_balance_to_earn_interest;
        product.account_fee = request.account_fee;
        product.maintenance_fee = request.maintenance_fee;
        product.withdrawal_fee = request.withdrawal_fee;
        product.deposit_fee = request.deposit_fee;
        product.inactive_account_fee = request.inactive_account_fee;
        product.reject_if_insufficient_funds = request.reject_if_insufficient_funds;
        product.is_active = request.is_active;

        self.service
            .update_savings_product(context, product)
            .await
            .map(|p| p.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteSavingsProductHandler<S: SavingsProductService> {
    service: Arc<S>,
}

impl<S: SavingsProductService> DeleteSavingsProductHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: SavingsProductService> CommandHandler<DeleteSavingsProductCommand>
    for DeleteSavingsProductHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteSavingsProduct";

    async fn handle(
        &self,
        request: DeleteSavingsProductCommand,
        context: &AuthorizedContext,
    ) -> Result<SavingsProductId, fineract_command::CommandError> {
        self.service
            .delete_savings_product(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: SavingsProductService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateSavingsProductHandler::new(service.clone()));
    dispatcher.register_handler(UpdateSavingsProductHandler::new(service.clone()));
    dispatcher.register_handler(DeleteSavingsProductHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(
            CreateSavingsProductCommand::COMMAND_TYPE,
            "CreateSavingsProduct"
        );
        assert_eq!(
            UpdateSavingsProductCommand::COMMAND_TYPE,
            "UpdateSavingsProduct"
        );
        assert_eq!(
            DeleteSavingsProductCommand::COMMAND_TYPE,
            "DeleteSavingsProduct"
        );
    }
}
