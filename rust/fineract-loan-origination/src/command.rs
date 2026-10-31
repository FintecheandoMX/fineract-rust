//! Command handlers for loan origination operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{LoanApplication, LoanApplicationStatus};
use crate::service::LoanOriginationService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitLoanApplicationCommand {
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub client_id: i64,
    pub requested_amount: String,
    pub currency: String,
    pub purpose: String,
    pub submitted_by: i64,
}

impl CommandRequest for SubmitLoanApplicationCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "SubmitLoanApplication";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLoanApplicationCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub client_id: i64,
    pub requested_amount: String,
    pub currency: String,
    pub purpose: String,
}

impl CommandRequest for UpdateLoanApplicationCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "UpdateLoanApplication";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteLoanApplicationCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteLoanApplicationCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "DeleteLoanApplication";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveLoanApplicationCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub reviewed_by: i64,
}

impl CommandRequest for ApproveLoanApplicationCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "ApproveLoanApplication";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectLoanApplicationCommand {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub reviewed_by: i64,
    pub rejection_reason: String,
}

impl CommandRequest for RejectLoanApplicationCommand {
    type Response = LoanId;
    const COMMAND_TYPE: &'static str = "RejectLoanApplication";
}

pub struct SubmitLoanApplicationHandler<S: LoanOriginationService> {
    service: Arc<S>,
}

impl<S: LoanOriginationService> SubmitLoanApplicationHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanOriginationService> CommandHandler<SubmitLoanApplicationCommand>
    for SubmitLoanApplicationHandler<S>
{
    const COMMAND_TYPE: &'static str = "SubmitLoanApplication";

    async fn handle(
        &self,
        request: SubmitLoanApplicationCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let requested_amount = rust_decimal::Decimal::from_str_exact(&request.requested_amount)
            .map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid amount".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;

        let app = LoanApplication::new(
            LoanId::new(1),
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            fineract_core::ids::ClientId::new(request.client_id),
            requested_amount,
            currency,
            request.purpose,
            fineract_core::ids::StaffId::new(request.submitted_by),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .submit_application(context, app)
            .await
            .map(|a| a.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateLoanApplicationHandler<S: LoanOriginationService> {
    service: Arc<S>,
}

impl<S: LoanOriginationService> UpdateLoanApplicationHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanOriginationService> CommandHandler<UpdateLoanApplicationCommand>
    for UpdateLoanApplicationHandler<S>
{
    const COMMAND_TYPE: &'static str = "UpdateLoanApplication";

    async fn handle(
        &self,
        request: UpdateLoanApplicationCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        let requested_amount = rust_decimal::Decimal::from_str_exact(&request.requested_amount)
            .map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid amount".to_string())
            })?;
        let currency =
            fineract_core::money::Currency::from_code(&request.currency).map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid currency code".to_string())
            })?;

        let app = LoanApplication::new(
            request.id,
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            fineract_core::ids::ClientId::new(request.client_id),
            requested_amount,
            currency,
            request.purpose,
            fineract_core::ids::StaffId::new(1),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .update_application(context, app)
            .await
            .map(|a| a.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteLoanApplicationHandler<S: LoanOriginationService> {
    service: Arc<S>,
}

impl<S: LoanOriginationService> DeleteLoanApplicationHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanOriginationService> CommandHandler<DeleteLoanApplicationCommand>
    for DeleteLoanApplicationHandler<S>
{
    const COMMAND_TYPE: &'static str = "DeleteLoanApplication";

    async fn handle(
        &self,
        request: DeleteLoanApplicationCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        self.service
            .delete_application(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct ApproveLoanApplicationHandler<S: LoanOriginationService> {
    service: Arc<S>,
}

impl<S: LoanOriginationService> ApproveLoanApplicationHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanOriginationService> CommandHandler<ApproveLoanApplicationCommand>
    for ApproveLoanApplicationHandler<S>
{
    const COMMAND_TYPE: &'static str = "ApproveLoanApplication";

    async fn handle(
        &self,
        request: ApproveLoanApplicationCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        self.service
            .update_status(
                context,
                request.id,
                LoanApplicationStatus::Approved,
                Some(fineract_core::ids::StaffId::new(request.reviewed_by)),
                None,
            )
            .await
            .map(|()| request.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct RejectLoanApplicationHandler<S: LoanOriginationService> {
    service: Arc<S>,
}

impl<S: LoanOriginationService> RejectLoanApplicationHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: LoanOriginationService> CommandHandler<RejectLoanApplicationCommand>
    for RejectLoanApplicationHandler<S>
{
    const COMMAND_TYPE: &'static str = "RejectLoanApplication";

    async fn handle(
        &self,
        request: RejectLoanApplicationCommand,
        context: &AuthorizedContext,
    ) -> Result<LoanId, fineract_command::CommandError> {
        self.service
            .update_status(
                context,
                request.id,
                LoanApplicationStatus::Rejected,
                Some(fineract_core::ids::StaffId::new(request.reviewed_by)),
                Some(request.rejection_reason),
            )
            .await
            .map(|()| request.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: LoanOriginationService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(SubmitLoanApplicationHandler::new(service.clone()));
    dispatcher.register_handler(UpdateLoanApplicationHandler::new(service.clone()));
    dispatcher.register_handler(DeleteLoanApplicationHandler::new(service.clone()));
    dispatcher.register_handler(ApproveLoanApplicationHandler::new(service.clone()));
    dispatcher.register_handler(RejectLoanApplicationHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(
            SubmitLoanApplicationCommand::COMMAND_TYPE,
            "SubmitLoanApplication"
        );
        assert_eq!(
            UpdateLoanApplicationCommand::COMMAND_TYPE,
            "UpdateLoanApplication"
        );
        assert_eq!(
            DeleteLoanApplicationCommand::COMMAND_TYPE,
            "DeleteLoanApplication"
        );
        assert_eq!(
            ApproveLoanApplicationCommand::COMMAND_TYPE,
            "ApproveLoanApplication"
        );
        assert_eq!(
            RejectLoanApplicationCommand::COMMAND_TYPE,
            "RejectLoanApplication"
        );
    }
}
