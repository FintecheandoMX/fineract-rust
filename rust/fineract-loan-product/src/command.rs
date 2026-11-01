//! Command handlers for loan product operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::LoanProduct;
use crate::service::LoanProductService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::LoanProductId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoanProductCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub principal: String,
    pub number_of_repayments: i32,
    pub repayment_every: i32,
    pub repayment_frequency_type: i32,
    pub amortization_type: i32,
    pub interest_rate_per_period: String,
    pub interest_type: i32,
    pub interest_calculation_period_type: i32,
    pub is_equal_amortization: bool,
    pub min_principal: String,
    pub max_principal: String,
    pub min_number_of_repayments: i32,
    pub max_number_of_repayments: i32,
    pub min_interest_rate_per_period: String,
    pub max_interest_rate_per_period: String,
    pub currency_code: String,
    pub currency_digits: i32,
    pub in_arrears_tolerance: i32,
    pub days_in_year_type: i32,
    pub days_in_month_type: i32,
    pub include_in_borrower_cycle: bool,
    pub use_borrower_cycle: bool,
    pub number_of_borrower_cycle: i32,
    pub is_active: bool,
}

impl CommandRequest for CreateLoanProductCommand {
    type Response = LoanProductId;
    const COMMAND_TYPE: &'static str = "CreateLoanProduct";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLoanProductCommand {
    pub id: LoanProductId,
    pub tenant_id: TenantId,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub principal: String,
    pub number_of_repayments: i32,
    pub repayment_every: i32,
    pub repayment_frequency_type: i32,
    pub amortization_type: i32,
    pub interest_rate_per_period: String,
    pub interest_type: i32,
    pub interest_calculation_period_type: i32,
    pub is_equal_amortization: bool,
    pub min_principal: String,
    pub max_principal: String,
    pub min_number_of_repayments: i32,
    pub max_number_of_repayments: i32,
    pub min_interest_rate_per_period: String,
    pub max_interest_rate_per_period: String,
    pub currency_code: String,
    pub currency_digits: i32,
    pub in_arrears_tolerance: i32,
    pub days_in_year_type: i32,
    pub days_in_month_type: i32,
    pub include_in_borrower_cycle: bool,
    pub use_borrower_cycle: bool,
    pub number_of_borrower_cycle: i32,
    pub is_active: bool,
}

impl CommandRequest for UpdateLoanProductCommand {
    type Response = LoanProductId;
    const COMMAND_TYPE: &'static str = "UpdateLoanProduct";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteLoanProductCommand {
    pub id: LoanProductId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteLoanProductCommand {
    type Response = LoanProductId;
    const COMMAND_TYPE: &'static str = "DeleteLoanProduct";
}

pub struct CreateLoanProductHandler<S: LoanProductService> {
    service: Arc<S>,
}

impl<S: LoanProductService> CreateLoanProductHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanProductService> CommandHandler<CreateLoanProductCommand>
    for CreateLoanProductHandler<S>
{
    const COMMAND_TYPE: &'static str = "CreateLoanProduct";

    async fn handle(
        &self,
        request: CreateLoanProductCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanProductId, fineract_command::CommandError> {
        let product = LoanProduct::new(
            LoanProductId::new(1),
            request.tenant_id,
            request.name,
            request.short_name,
            request.description,
            request.principal,
            request.number_of_repayments,
            request.repayment_every,
            request.repayment_frequency_type,
            request.amortization_type,
            request.interest_rate_per_period,
            request.interest_type,
            request.interest_calculation_period_type,
            request.currency_code,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut product = product;
        product.is_equal_amortization = request.is_equal_amortization;
        product.min_principal = request.min_principal;
        product.max_principal = request.max_principal;
        product.min_number_of_repayments = request.min_number_of_repayments;
        product.max_number_of_repayments = request.max_number_of_repayments;
        product.min_interest_rate_per_period = request.min_interest_rate_per_period;
        product.max_interest_rate_per_period = request.max_interest_rate_per_period;
        product.currency_digits = request.currency_digits;
        product.in_arrears_tolerance = request.in_arrears_tolerance;
        product.days_in_year_type = request.days_in_year_type;
        product.days_in_month_type = request.days_in_month_type;
        product.include_in_borrower_cycle = request.include_in_borrower_cycle;
        product.use_borrower_cycle = request.use_borrower_cycle;
        product.number_of_borrower_cycle = request.number_of_borrower_cycle;
        product.is_active = request.is_active;

        self.service
            .create_loan_product(context, product)
            .await
            .map(|p| p.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateLoanProductHandler<S: LoanProductService> {
    service: Arc<S>,
}

impl<S: LoanProductService> UpdateLoanProductHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanProductService> CommandHandler<UpdateLoanProductCommand>
    for UpdateLoanProductHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateLoanProduct";

    async fn handle(
        &self,
        request: UpdateLoanProductCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanProductId, fineract_command::CommandError> {
        let product = LoanProduct::new(
            request.id,
            request.tenant_id,
            request.name,
            request.short_name,
            request.description,
            request.principal,
            request.number_of_repayments,
            request.repayment_every,
            request.repayment_frequency_type,
            request.amortization_type,
            request.interest_rate_per_period,
            request.interest_type,
            request.interest_calculation_period_type,
            request.currency_code,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut product = product;
        product.is_equal_amortization = request.is_equal_amortization;
        product.min_principal = request.min_principal;
        product.max_principal = request.max_principal;
        product.min_number_of_repayments = request.min_number_of_repayments;
        product.max_number_of_repayments = request.max_number_of_repayments;
        product.min_interest_rate_per_period = request.min_interest_rate_per_period;
        product.max_interest_rate_per_period = request.max_interest_rate_per_period;
        product.currency_digits = request.currency_digits;
        product.in_arrears_tolerance = request.in_arrears_tolerance;
        product.days_in_year_type = request.days_in_year_type;
        product.days_in_month_type = request.days_in_month_type;
        product.include_in_borrower_cycle = request.include_in_borrower_cycle;
        product.use_borrower_cycle = request.use_borrower_cycle;
        product.number_of_borrower_cycle = request.number_of_borrower_cycle;
        product.is_active = request.is_active;

        self.service
            .update_loan_product(context, product)
            .await
            .map(|p| p.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteLoanProductHandler<S: LoanProductService> {
    service: Arc<S>,
}

impl<S: LoanProductService> DeleteLoanProductHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanProductService> CommandHandler<DeleteLoanProductCommand>
    for DeleteLoanProductHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteLoanProduct";

    async fn handle(
        &self,
        request: DeleteLoanProductCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanProductId, fineract_command::CommandError> {
        self.service
            .delete_loan_product(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: LoanProductService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateLoanProductHandler::new(service.clone()));
    dispatcher.register_handler(UpdateLoanProductHandler::new(service.clone()));
    dispatcher.register_handler(DeleteLoanProductHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateLoanProductCommand::COMMAND_TYPE, "CreateLoanProduct");
        assert_eq!(UpdateLoanProductCommand::COMMAND_TYPE, "UpdateLoanProduct");
        assert_eq!(DeleteLoanProductCommand::COMMAND_TYPE, "DeleteLoanProduct");
    }
}
