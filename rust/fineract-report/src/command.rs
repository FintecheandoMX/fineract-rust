//! Command handlers for report operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{Report, ReportFormat, ReportStatus, ReportType};
use crate::service::ReportService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::ReportId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub report_format: String,
    pub parameters: Option<serde_json::Value>,
}

impl CommandRequest for CreateReportCommand {
    type Response = ReportId;
    const COMMAND_TYPE: &'static str = "CreateReport";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportCommand {
    pub id: ReportId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub report_format: String,
    pub parameters: Option<serde_json::Value>,
    pub status: String,
    pub file_path: Option<String>,
    pub generated_by: Option<i64>,
    pub generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

impl CommandRequest for UpdateReportCommand {
    type Response = ReportId;
    const COMMAND_TYPE: &'static str = "UpdateReport";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteReportCommand {
    pub id: ReportId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteReportCommand {
    type Response = ReportId;
    const COMMAND_TYPE: &'static str = "DeleteReport";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkReportGeneratingCommand {
    pub id: ReportId,
    pub tenant_id: TenantId,
}

impl CommandRequest for MarkReportGeneratingCommand {
    type Response = ReportId;
    const COMMAND_TYPE: &'static str = "MarkReportGenerating";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkReportCompletedCommand {
    pub id: ReportId,
    pub tenant_id: TenantId,
    pub file_path: String,
}

impl CommandRequest for MarkReportCompletedCommand {
    type Response = ReportId;
    const COMMAND_TYPE: &'static str = "MarkReportCompleted";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkReportFailedCommand {
    pub id: ReportId,
    pub tenant_id: TenantId,
    pub error_message: String,
}

impl CommandRequest for MarkReportFailedCommand {
    type Response = ReportId;
    const COMMAND_TYPE: &'static str = "MarkReportFailed";
}

fn parse_report_type(s: &str) -> Result<ReportType, String> {
    match s.to_lowercase().as_str() {
        "portfolio" => Ok(ReportType::Portfolio),
        "savings" => Ok(ReportType::Savings),
        "loan" => Ok(ReportType::Loan),
        "client" => Ok(ReportType::Client),
        "accounting" => Ok(ReportType::Accounting),
        "delinquency" => Ok(ReportType::Delinquency),
        "disbursement" => Ok(ReportType::Disbursement),
        "repayment" => Ok(ReportType::Repayment),
        "aging" => Ok(ReportType::Aging),
        "custom" => Ok(ReportType::Custom),
        _ => Err(format!("invalid report_type: {s}")),
    }
}

fn parse_report_format(s: &str) -> Result<ReportFormat, String> {
    match s.to_lowercase().as_str() {
        "pdf" => Ok(ReportFormat::Pdf),
        "excel" => Ok(ReportFormat::Excel),
        "csv" => Ok(ReportFormat::Csv),
        "html" => Ok(ReportFormat::Html),
        "json" => Ok(ReportFormat::Json),
        _ => Err(format!("invalid report_format: {s}")),
    }
}

fn parse_report_status(s: &str) -> Result<ReportStatus, String> {
    match s.to_lowercase().as_str() {
        "pending" => Ok(ReportStatus::Pending),
        "generating" => Ok(ReportStatus::Generating),
        "completed" => Ok(ReportStatus::Completed),
        "failed" => Ok(ReportStatus::Failed),
        "cancelled" => Ok(ReportStatus::Cancelled),
        _ => Err(format!("invalid report_status: {s}")),
    }
}

pub struct CreateReportHandler<S: ReportService> {
    service: Arc<S>,
}

impl<S: ReportService> CreateReportHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ReportService> CommandHandler<CreateReportCommand> for CreateReportHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateReport";

    async fn handle(
        &self,
        request: CreateReportCommand,
        context: &AuthorizedContext,
    ) -> Result<ReportId, fineract_command::CommandError> {
        let report_type = parse_report_type(&request.report_type)
            .map_err(fineract_command::CommandError::HandlerError)?;
        let report_format = parse_report_format(&request.report_format)
            .map_err(fineract_command::CommandError::HandlerError)?;

        let report = Report::new(
            ReportId::new(1),
            request.tenant_id,
            request.name,
            request.description,
            report_type,
            report_format,
            request.parameters,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_report(context, report)
            .await
            .map(|r| r.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateReportHandler<S: ReportService> {
    service: Arc<S>,
}

impl<S: ReportService> UpdateReportHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ReportService> CommandHandler<UpdateReportCommand> for UpdateReportHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateReport";

    async fn handle(
        &self,
        request: UpdateReportCommand,
        context: &AuthorizedContext,
    ) -> Result<ReportId, fineract_command::CommandError> {
        let report_type = parse_report_type(&request.report_type)
            .map_err(fineract_command::CommandError::HandlerError)?;
        let report_format = parse_report_format(&request.report_format)
            .map_err(fineract_command::CommandError::HandlerError)?;
        let status = parse_report_status(&request.status)
            .map_err(fineract_command::CommandError::HandlerError)?;

        let mut report = Report::new(
            request.id,
            request.tenant_id,
            request.name,
            request.description,
            report_type,
            report_format,
            request.parameters,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        report.status = status;
        report.file_path = request.file_path;
        report.generated_by = request.generated_by;
        report.generated_at = request.generated_at;
        report.error_message = request.error_message;

        self.service
            .update_report(context, report)
            .await
            .map(|r| r.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteReportHandler<S: ReportService> {
    service: Arc<S>,
}

impl<S: ReportService> DeleteReportHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ReportService> CommandHandler<DeleteReportCommand> for DeleteReportHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteReport";

    async fn handle(
        &self,
        request: DeleteReportCommand,
        context: &AuthorizedContext,
    ) -> Result<ReportId, fineract_command::CommandError> {
        self.service
            .delete_report(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct MarkReportGeneratingHandler<S: ReportService> {
    service: Arc<S>,
}

impl<S: ReportService> MarkReportGeneratingHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ReportService> CommandHandler<MarkReportGeneratingCommand>
    for MarkReportGeneratingHandler<S>
{
    const COMMAND_TYPE: &'static str = "MarkReportGenerating";

    async fn handle(
        &self,
        request: MarkReportGeneratingCommand,
        context: &AuthorizedContext,
    ) -> Result<ReportId, fineract_command::CommandError> {
        self.service
            .mark_report_generating(context, request.id)
            .await
            .map(|r| r.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct MarkReportCompletedHandler<S: ReportService> {
    service: Arc<S>,
}

impl<S: ReportService> MarkReportCompletedHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ReportService> CommandHandler<MarkReportCompletedCommand>
    for MarkReportCompletedHandler<S>
{
    const COMMAND_TYPE: &'static str = "MarkReportCompleted";

    async fn handle(
        &self,
        request: MarkReportCompletedCommand,
        context: &AuthorizedContext,
    ) -> Result<ReportId, fineract_command::CommandError> {
        self.service
            .mark_report_completed(context, request.id, request.file_path)
            .await
            .map(|r| r.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct MarkReportFailedHandler<S: ReportService> {
    service: Arc<S>,
}

impl<S: ReportService> MarkReportFailedHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ReportService> CommandHandler<MarkReportFailedCommand> for MarkReportFailedHandler<S> {
    const COMMAND_TYPE: &'static str = "MarkReportFailed";

    async fn handle(
        &self,
        request: MarkReportFailedCommand,
        context: &AuthorizedContext,
    ) -> Result<ReportId, fineract_command::CommandError> {
        self.service
            .mark_report_failed(context, request.id, request.error_message)
            .await
            .map(|r| r.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: ReportService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateReportHandler::new(service.clone()));
    dispatcher.register_handler(UpdateReportHandler::new(service.clone()));
    dispatcher.register_handler(DeleteReportHandler::new(service.clone()));
    dispatcher.register_handler(MarkReportGeneratingHandler::new(service.clone()));
    dispatcher.register_handler(MarkReportCompletedHandler::new(service.clone()));
    dispatcher.register_handler(MarkReportFailedHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateReportCommand::COMMAND_TYPE, "CreateReport");
        assert_eq!(UpdateReportCommand::COMMAND_TYPE, "UpdateReport");
        assert_eq!(DeleteReportCommand::COMMAND_TYPE, "DeleteReport");
        assert_eq!(
            MarkReportGeneratingCommand::COMMAND_TYPE,
            "MarkReportGenerating"
        );
        assert_eq!(
            MarkReportCompletedCommand::COMMAND_TYPE,
            "MarkReportCompleted"
        );
        assert_eq!(MarkReportFailedCommand::COMMAND_TYPE, "MarkReportFailed");
    }
}
