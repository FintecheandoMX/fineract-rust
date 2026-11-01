//! Command handlers for staff operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Staff;
use crate::service::StaffService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStaffCommand {
    pub tenant_id: TenantId,
    pub office_id: Option<i64>,
    pub firstname: String,
    pub lastname: String,
    pub username: String,
    pub email: Option<String>,
    pub mobile_no: Option<String>,
    pub is_loan_officer: bool,
    pub is_active: bool,
    pub external_id: Option<String>,
}

impl CommandRequest for CreateStaffCommand {
    type Response = StaffId;
    const COMMAND_TYPE: &'static str = "CreateStaff";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStaffCommand {
    pub id: StaffId,
    pub tenant_id: TenantId,
    pub office_id: Option<i64>,
    pub firstname: String,
    pub lastname: String,
    pub username: String,
    pub email: Option<String>,
    pub mobile_no: Option<String>,
    pub is_loan_officer: bool,
    pub is_active: bool,
    pub external_id: Option<String>,
}

impl CommandRequest for UpdateStaffCommand {
    type Response = StaffId;
    const COMMAND_TYPE: &'static str = "UpdateStaff";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteStaffCommand {
    pub id: StaffId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteStaffCommand {
    type Response = StaffId;
    const COMMAND_TYPE: &'static str = "DeleteStaff";
}

pub struct CreateStaffHandler<S: StaffService> {
    service: Arc<S>,
}

impl<S: StaffService> CreateStaffHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: StaffService> CommandHandler<CreateStaffCommand> for CreateStaffHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateStaff";

    async fn handle(
        &self,
        request: CreateStaffCommand,
        context: &AuthorizedContext,
    ) -> Result<StaffId, fineract_command::CommandError> {
        let staff = Staff::new(
            StaffId::new(1),
            request.tenant_id,
            request.firstname,
            request.lastname,
            request.username,
            request.office_id.map(OfficeId::new),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut staff = staff;
        staff.email = request.email;
        staff.mobile_no = request.mobile_no;
        staff.is_loan_officer = request.is_loan_officer;
        staff.is_active = request.is_active;
        staff.external_id = request.external_id;

        self.service
            .create_staff(context, staff)
            .await
            .map(|s| s.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateStaffHandler<S: StaffService> {
    service: Arc<S>,
}

impl<S: StaffService> UpdateStaffHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: StaffService> CommandHandler<UpdateStaffCommand> for UpdateStaffHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateStaff";

    async fn handle(
        &self,
        request: UpdateStaffCommand,
        context: &AuthorizedContext,
    ) -> Result<StaffId, fineract_command::CommandError> {
        let mut staff = Staff::new(
            request.id,
            request.tenant_id,
            request.firstname,
            request.lastname,
            request.username,
            request.office_id.map(OfficeId::new),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        staff.email = request.email;
        staff.mobile_no = request.mobile_no;
        staff.is_loan_officer = request.is_loan_officer;
        staff.is_active = request.is_active;
        staff.external_id = request.external_id;

        self.service
            .update_staff(context, staff)
            .await
            .map(|s| s.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteStaffHandler<S: StaffService> {
    service: Arc<S>,
}

impl<S: StaffService> DeleteStaffHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: StaffService> CommandHandler<DeleteStaffCommand> for DeleteStaffHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteStaff";

    async fn handle(
        &self,
        request: DeleteStaffCommand,
        context: &AuthorizedContext,
    ) -> Result<StaffId, fineract_command::CommandError> {
        self.service
            .delete_staff(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: StaffService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateStaffHandler::new(service.clone()));
    dispatcher.register_handler(UpdateStaffHandler::new(service.clone()));
    dispatcher.register_handler(DeleteStaffHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateStaffCommand::COMMAND_TYPE, "CreateStaff");
        assert_eq!(UpdateStaffCommand::COMMAND_TYPE, "UpdateStaff");
        assert_eq!(DeleteStaffCommand::COMMAND_TYPE, "DeleteStaff");
    }
}
