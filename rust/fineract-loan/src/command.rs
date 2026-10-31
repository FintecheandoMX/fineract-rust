//! Command handlers for loan operations.
//! All write operations go through the command pipeline (idempotency, maker-checker, audit).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{Loan, LoanStatus, LoanTransaction, LoanTransactionType, LoanType};
use crate::service::LoanService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::LoanTransactionId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoanCommand {
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub client_id: i64,
    pub loan_type: String,
    pub principal: String,
    pub currency: String,
    pub interest_rate: String,
    pub term_count: i32,
    pub term_period: String,
    pub disbursement_date: String,
    pub external_id: Option<String>,
}

impl CommandRequest for CreateLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "CreateLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLoanCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub client_id: i64,
    pub loan_type: String,
    pub principal: String,
    pub currency: String,
    pub interest_rate: String,
    pub term_count: i32,
    pub term_period: String,
    pub disbursement_date: String,
    pub external_id: Option<String>,
}

impl CommandRequest for UpdateLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "UpdateLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteLoanCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteLoanCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "DeleteLoan";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLoanStatusCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub status: String,
}

impl CommandRequest for UpdateLoanStatusCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "UpdateLoanStatus";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordTransactionCommand {
    pub loan_id: LoanId,
    pub tenant_id: TenantId,
    pub amount: String,
    pub transaction_type: String,
    pub reference: Option<String>,
}

impl CommandRequest for RecordTransactionCommand {
    type Response = LoanTransactionId;
    const COMMAND_TYPE: &'static str = "RecordTransaction";
}

pub struct CreateLoanHandler<S: LoanService> {
    service: Arc<S>,
}

impl<S: LoanService> CreateLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanService> CommandHandler<CreateLoanCommand> for CreateLoanHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateLoan";

    async fn handle(
        &self,
        request: CreateLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let loan_type = match request.loan_type.as_str() {
            "Individual" => LoanType::Individual,
            "Group" => LoanType::Group,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid loan type".to_string(),
                ))
            }
        };
        let principal =
            rust_decimal::Decimal::from_str_exact(&request.principal).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid principal".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let interest_rate =
            rust_decimal::Decimal::from_str_exact(&request.interest_rate).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid interest rate".to_string())
            })?;
        let term_period = match request.term_period.as_str() {
            "Days" => crate::domain::LoanTermPeriod::Days,
            "Weeks" => crate::domain::LoanTermPeriod::Weeks,
            "Months" => crate::domain::LoanTermPeriod::Months,
            "Years" => crate::domain::LoanTermPeriod::Years,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid term period".to_string(),
                ))
            }
        };
        let disbursement_date = chrono::NaiveDate::parse_from_str(
            &request.disbursement_date,
            "%Y-%m-%d",
        )
        .map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid disbursement date".to_string())
        })?;

        let loan = Loan::new(
            LoanId::new(1),
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            fineract_core::ids::ClientId::new(request.client_id),
            loan_type,
            principal,
            currency,
            interest_rate,
            request.term_count,
            term_period,
            disbursement_date,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_loan(context, loan)
            .await
            .map(|l| l.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateLoanHandler<S: LoanService> {
    service: Arc<S>,
}

impl<S: LoanService> UpdateLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanService> CommandHandler<UpdateLoanCommand> for UpdateLoanHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateLoan";

    async fn handle(
        &self,
        request: UpdateLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let loan_type = match request.loan_type.as_str() {
            "Individual" => LoanType::Individual,
            "Group" => LoanType::Group,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid loan type".to_string(),
                ))
            }
        };
        let principal =
            rust_decimal::Decimal::from_str_exact(&request.principal).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid principal".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;
        let interest_rate =
            rust_decimal::Decimal::from_str_exact(&request.interest_rate).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid interest rate".to_string())
            })?;
        let term_period = match request.term_period.as_str() {
            "Days" => crate::domain::LoanTermPeriod::Days,
            "Weeks" => crate::domain::LoanTermPeriod::Weeks,
            "Months" => crate::domain::LoanTermPeriod::Months,
            "Years" => crate::domain::LoanTermPeriod::Years,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid term period".to_string(),
                ))
            }
        };
        let disbursement_date = chrono::NaiveDate::parse_from_str(
            &request.disbursement_date,
            "%Y-%m-%d",
        )
        .map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid disbursement date".to_string())
        })?;

        let mut loan = Loan::new(
            request.id,
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            fineract_core::ids::ClientId::new(request.client_id),
            loan_type,
            principal,
            currency,
            interest_rate,
            request.term_count,
            term_period,
            disbursement_date,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        loan.external_id = request.external_id;

        self.service
            .update_loan(context, loan)
            .await
            .map(|l| l.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteLoanHandler<S: LoanService> {
    service: Arc<S>,
}

impl<S: LoanService> DeleteLoanHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanService> CommandHandler<DeleteLoanCommand> for DeleteLoanHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteLoan";

    async fn handle(
        &self,
        request: DeleteLoanCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        self.service
            .delete_loan(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateLoanStatusHandler<S: LoanService> {
    service: Arc<S>,
}

impl<S: LoanService> UpdateLoanStatusHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanService> CommandHandler<UpdateLoanStatusCommand> for UpdateLoanStatusHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateLoanStatus";

    async fn handle(
        &self,
        request: UpdateLoanStatusCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let status = match request.status.as_str() {
            "PendingApproval" => LoanStatus::PendingApproval,
            "Approved" => LoanStatus::Approved,
            "Active" => LoanStatus::Active,
            "Overpaid" => LoanStatus::Overpaid,
            "Closed" => LoanStatus::Closed,
            "Rejected" => LoanStatus::Rejected,
            "WrittenOff" => LoanStatus::WrittenOff,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid loan status".to_string(),
                ))
            }
        };
        self.service
            .update_loan_status(context, request.id, status)
            .await
            .map(|()| request.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct RecordTransactionHandler<S: LoanService> {
    service: Arc<S>,
}

impl<S: LoanService> RecordTransactionHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanService> CommandHandler<RecordTransactionCommand> for RecordTransactionHandler<S> {
    const COMMAND_TYPE: &'static str = "RecordTransaction";

    async fn handle(
        &self,
        request: RecordTransactionCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanTransactionId, fineract_command::CommandError> {
        let amount = rust_decimal::Decimal::from_str_exact(&request.amount).map_err(|_| {
            fineract_command::CommandError::HandlerError("invalid amount".to_string())
        })?;
        let transaction_type = match request.transaction_type.as_str() {
            "Disbursement" => LoanTransactionType::Disbursement,
            "Repayment" => LoanTransactionType::Repayment,
            "Fee" => LoanTransactionType::Fee,
            "Interest" => LoanTransactionType::Interest,
            "Reversal" => LoanTransactionType::Reversal,
            "WriteOff" => LoanTransactionType::WriteOff,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid transaction type".to_string(),
                ))
            }
        };
        let tx = LoanTransaction {
            id: LoanTransactionId::new(1),
            loan_id: request.loan_id,
            tenant_id: request.tenant_id,
            amount,
            transaction_type,
            transaction_date: chrono::Utc::now(),
            reference: request.reference,
            active: true,
        };
        self.service
            .record_transaction(context, tx)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: LoanService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateLoanHandler::new(service.clone()));
    dispatcher.register_handler(UpdateLoanHandler::new(service.clone()));
    dispatcher.register_handler(DeleteLoanHandler::new(service.clone()));
    dispatcher.register_handler(UpdateLoanStatusHandler::new(service.clone()));
    dispatcher.register_handler(RecordTransactionHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateLoanCommand::COMMAND_TYPE, "CreateLoan");
        assert_eq!(UpdateLoanCommand::COMMAND_TYPE, "UpdateLoan");
        assert_eq!(DeleteLoanCommand::COMMAND_TYPE, "DeleteLoan");
        assert_eq!(UpdateLoanStatusCommand::COMMAND_TYPE, "UpdateLoanStatus");
        assert_eq!(RecordTransactionCommand::COMMAND_TYPE, "RecordTransaction");
        assert_ne!(
            CreateLoanCommand::COMMAND_TYPE,
            UpdateLoanCommand::COMMAND_TYPE
        );
    }
}
