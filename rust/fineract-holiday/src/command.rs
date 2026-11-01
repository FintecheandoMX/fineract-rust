//! Command handlers for holiday operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Holiday;
use crate::service::HolidayService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::HolidayId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHolidayCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub holiday_date: String,
    pub is_recurring: bool,
    pub active: bool,
}

impl CommandRequest for CreateHolidayCommand {
    type Response = HolidayId;
    const COMMAND_TYPE: &'static str = "CreateHoliday";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHolidayCommand {
    pub id: HolidayId,
    pub tenant_id: TenantId,
    pub name: String,
    pub holiday_date: String,
    pub is_recurring: bool,
    pub active: bool,
}

impl CommandRequest for UpdateHolidayCommand {
    type Response = HolidayId;
    const COMMAND_TYPE: &'static str = "UpdateHoliday";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteHolidayCommand {
    pub id: HolidayId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteHolidayCommand {
    type Response = HolidayId;
    const COMMAND_TYPE: &'static str = "DeleteHoliday";
}

pub struct CreateHolidayHandler<S: HolidayService> {
    service: Arc<S>,
}

impl<S: HolidayService> CreateHolidayHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: HolidayService> CommandHandler<CreateHolidayCommand> for CreateHolidayHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateHoliday";

    async fn handle(
        &self,
        request: CreateHolidayCommand,
        context: &AuthorizedContext,
    ) -> Result<HolidayId, fineract_command::CommandError> {
        let holiday_date = chrono::NaiveDate::parse_from_str(&request.holiday_date, "%Y-%m-%d")
            .map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid holiday date".to_string())
            })?;

        let holiday = Holiday::new(
            HolidayId::new(1),
            request.tenant_id,
            request.name,
            holiday_date,
            request.is_recurring,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut holiday = holiday;
        holiday.active = request.active;

        self.service
            .create_holiday(context, holiday)
            .await
            .map(|h| h.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateHolidayHandler<S: HolidayService> {
    service: Arc<S>,
}

impl<S: HolidayService> UpdateHolidayHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: HolidayService> CommandHandler<UpdateHolidayCommand> for UpdateHolidayHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateHoliday";

    async fn handle(
        &self,
        request: UpdateHolidayCommand,
        context: &AuthorizedContext,
    ) -> Result<HolidayId, fineract_command::CommandError> {
        let holiday_date = chrono::NaiveDate::parse_from_str(&request.holiday_date, "%Y-%m-%d")
            .map_err(|_| {
                fineract_command::CommandError::HandlerError("invalid holiday date".to_string())
            })?;

        let holiday = Holiday::new(
            request.id,
            request.tenant_id,
            request.name,
            holiday_date,
            request.is_recurring,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut holiday = holiday;
        holiday.active = request.active;

        self.service
            .update_holiday(context, holiday)
            .await
            .map(|h| h.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteHolidayHandler<S: HolidayService> {
    service: Arc<S>,
}

impl<S: HolidayService> DeleteHolidayHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: HolidayService> CommandHandler<DeleteHolidayCommand> for DeleteHolidayHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteHoliday";

    async fn handle(
        &self,
        request: DeleteHolidayCommand,
        context: &AuthorizedContext,
    ) -> Result<HolidayId, fineract_command::CommandError> {
        self.service
            .delete_holiday(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: HolidayService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateHolidayHandler::new(service.clone()));
    dispatcher.register_handler(UpdateHolidayHandler::new(service.clone()));
    dispatcher.register_handler(DeleteHolidayHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateHolidayCommand::COMMAND_TYPE, "CreateHoliday");
        assert_eq!(UpdateHolidayCommand::COMMAND_TYPE, "UpdateHoliday");
        assert_eq!(DeleteHolidayCommand::COMMAND_TYPE, "DeleteHoliday");
    }
}
