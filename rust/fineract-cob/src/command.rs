//! Command handlers for COB job operations.
//! All write operations go through the command pipeline (idempotency, maker-checker, audit).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{Job, JobType};
use crate::service::JobService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::JobId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub active: bool,
}

impl CommandRequest for CreateJobCommand {
    type Response = JobId;
    const COMMAND_TYPE: &'static str = "CreateJob";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJobCommand {
    pub id: JobId,
    pub tenant_id: TenantId,
    pub name: String,
    pub job_type: String,
    pub cron_expression: String,
    pub active: bool,
}

impl CommandRequest for UpdateJobCommand {
    type Response = JobId;
    const COMMAND_TYPE: &'static str = "UpdateJob";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableJobCommand {
    pub id: JobId,
    pub tenant_id: TenantId,
}

impl CommandRequest for EnableJobCommand {
    type Response = JobId;
    const COMMAND_TYPE: &'static str = "EnableJob";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisableJobCommand {
    pub id: JobId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DisableJobCommand {
    type Response = JobId;
    const COMMAND_TYPE: &'static str = "DisableJob";
}

pub struct CreateJobHandler<S: JobService> {
    service: Arc<S>,
}

impl<S: JobService> CreateJobHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: JobService> CommandHandler<CreateJobCommand> for CreateJobHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateJob";

    async fn handle(
        &self,
        request: CreateJobCommand,
        context: &AuthorizedContext,
    ) -> Result<JobId, fineract_command::CommandError> {
        let job_type = match request.job_type.as_str() {
            "CloseOfBusiness" => JobType::CloseOfBusiness,
            "InterestAccrual" => JobType::InterestAccrual,
            "ChargeApplication" => JobType::ChargeApplication,
            "DataArchival" => JobType::DataArchival,
            s => JobType::Custom(s.to_string()),
        };
        let job = Job::new(
            JobId::new(1), // TODO: server-side ID generation
            request.tenant_id,
            request.name,
            job_type,
            request.cron_expression,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_job(context, job)
            .await
            .map(|j| j.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateJobHandler<S: JobService> {
    service: Arc<S>,
}

impl<S: JobService> UpdateJobHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: JobService> CommandHandler<UpdateJobCommand> for UpdateJobHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateJob";

    async fn handle(
        &self,
        request: UpdateJobCommand,
        context: &AuthorizedContext,
    ) -> Result<JobId, fineract_command::CommandError> {
        let job_type = match request.job_type.as_str() {
            "CloseOfBusiness" => JobType::CloseOfBusiness,
            "InterestAccrual" => JobType::InterestAccrual,
            "ChargeApplication" => JobType::ChargeApplication,
            "DataArchival" => JobType::DataArchival,
            s => JobType::Custom(s.to_string()),
        };
        let mut job = Job::new(
            request.id,
            request.tenant_id,
            request.name,
            job_type,
            request.cron_expression,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        job.active = request.active;
        self.service
            .update_job(context, job)
            .await
            .map(|j| j.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct EnableJobHandler<S: JobService> {
    service: Arc<S>,
}

impl<S: JobService> EnableJobHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: JobService> CommandHandler<EnableJobCommand> for EnableJobHandler<S> {
    const COMMAND_TYPE: &'static str = "EnableJob";

    async fn handle(
        &self,
        request: EnableJobCommand,
        context: &AuthorizedContext,
    ) -> Result<JobId, fineract_command::CommandError> {
        self.service
            .enable_job(context, request.id)
            .await
            .map(|()| request.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DisableJobHandler<S: JobService> {
    service: Arc<S>,
}

impl<S: JobService> DisableJobHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: JobService> CommandHandler<DisableJobCommand> for DisableJobHandler<S> {
    const COMMAND_TYPE: &'static str = "DisableJob";

    async fn handle(
        &self,
        request: DisableJobCommand,
        context: &AuthorizedContext,
    ) -> Result<JobId, fineract_command::CommandError> {
        self.service
            .disable_job(context, request.id)
            .await
            .map(|()| request.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: JobService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateJobHandler::new(service.clone()));
    dispatcher.register_handler(UpdateJobHandler::new(service.clone()));
    dispatcher.register_handler(EnableJobHandler::new(service.clone()));
    dispatcher.register_handler(DisableJobHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateJobCommand::COMMAND_TYPE, "CreateJob");
        assert_eq!(UpdateJobCommand::COMMAND_TYPE, "UpdateJob");
        assert_eq!(EnableJobCommand::COMMAND_TYPE, "EnableJob");
        assert_eq!(DisableJobCommand::COMMAND_TYPE, "DisableJob");
        assert_ne!(
            CreateJobCommand::COMMAND_TYPE,
            UpdateJobCommand::COMMAND_TYPE
        );
    }
}
