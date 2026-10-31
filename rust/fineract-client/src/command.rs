//! Command handlers for client operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::{Client, ClientStatus, ClientType, Group, GroupStatus};
use crate::service::ClientService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::ClientId;
use fineract_core::ids::GroupId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientCommand {
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub client_type: String,
    pub display_name: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
    pub mobile_no: Option<String>,
    pub email: Option<String>,
    pub staff_id: Option<i64>,
    pub external_id: Option<String>,
}

impl CommandRequest for CreateClientCommand {
    type Response = ClientId;
    const COMMAND_TYPE: &'static str = "CreateClient";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientCommand {
    pub id: ClientId,
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub client_type: String,
    pub display_name: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
    pub mobile_no: Option<String>,
    pub email: Option<String>,
    pub staff_id: Option<i64>,
    pub external_id: Option<String>,
    pub active: bool,
}

impl CommandRequest for UpdateClientCommand {
    type Response = ClientId;
    const COMMAND_TYPE: &'static str = "UpdateClient";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteClientCommand {
    pub id: ClientId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteClientCommand {
    type Response = ClientId;
    const COMMAND_TYPE: &'static str = "DeleteClient";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateClientCommand {
    pub id: ClientId,
    pub tenant_id: TenantId,
}

impl CommandRequest for ActivateClientCommand {
    type Response = ClientId;
    const COMMAND_TYPE: &'static str = "ActivateClient";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupCommand {
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub name: String,
    pub display_name: String,
    pub staff_id: Option<i64>,
    pub external_id: Option<String>,
    pub active: bool,
}

impl CommandRequest for CreateGroupCommand {
    type Response = GroupId;
    const COMMAND_TYPE: &'static str = "CreateGroup";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupCommand {
    pub id: GroupId,
    pub tenant_id: TenantId,
    pub office_id: i64,
    pub name: String,
    pub display_name: String,
    pub staff_id: Option<i64>,
    pub external_id: Option<String>,
    pub active: bool,
}

impl CommandRequest for UpdateGroupCommand {
    type Response = GroupId;
    const COMMAND_TYPE: &'static str = "UpdateGroup";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteGroupCommand {
    pub id: GroupId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteGroupCommand {
    type Response = GroupId;
    const COMMAND_TYPE: &'static str = "DeleteGroup";
}

pub struct CreateClientHandler<S: ClientService> {
    service: Arc<S>,
}

impl<S: ClientService> CreateClientHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ClientService> CommandHandler<CreateClientCommand> for CreateClientHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateClient";

    async fn handle(
        &self,
        request: CreateClientCommand,
        context: &AuthorizedContext,
    ) -> Result<ClientId, fineract_command::CommandError> {
        let client_type = match request.client_type.as_str() {
            "Individual" => crate::domain::ClientType::Individual,
            "Group" => crate::domain::ClientType::Group,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid client type".to_string(),
                ))
            }
        };
        let client = Client::new(
            ClientId::new(1),
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            client_type,
            request.display_name,
            request.staff_id.map(fineract_core::ids::StaffId::new),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut client = client;
        client.firstname = request.firstname;
        client.lastname = request.lastname;
        client.date_of_birth = request
            .date_of_birth
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        client.gender = request.gender;
        client.mobile_no = request.mobile_no;
        client.email = request.email;
        client.staff_id = request.staff_id.map(fineract_core::ids::StaffId::new);
        client.external_id = request.external_id;
        client.status = crate::domain::ClientStatus::Pending;
        client.active = false;

        self.service
            .create_client(context, client)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateClientHandler<S: ClientService> {
    service: Arc<S>,
}

impl<S: ClientService> UpdateClientHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ClientService> CommandHandler<UpdateClientCommand> for UpdateClientHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateClient";

    async fn handle(
        &self,
        request: UpdateClientCommand,
        context: &AuthorizedContext,
    ) -> Result<ClientId, fineract_command::CommandError> {
        let client_type = match request.client_type.as_str() {
            "Individual" => crate::domain::ClientType::Individual,
            "Group" => crate::domain::ClientType::Group,
            _ => {
                return Err(fineract_command::CommandError::HandlerError(
                    "invalid client type".to_string(),
                ))
            }
        };
        let mut client = Client::new(
            request.id,
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            client_type,
            request.display_name,
            request.staff_id.map(fineract_core::ids::StaffId::new),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        client.firstname = request.firstname;
        client.lastname = request.lastname;
        client.date_of_birth = request
            .date_of_birth
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        client.gender = request.gender;
        client.mobile_no = request.mobile_no;
        client.email = request.email;
        client.staff_id = request.staff_id.map(fineract_core::ids::StaffId::new);
        client.external_id = request.external_id;
        client.active = request.active;

        self.service
            .update_client(context, client)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteClientHandler<S: ClientService> {
    service: Arc<S>,
}

impl<S: ClientService> DeleteClientHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ClientService> CommandHandler<DeleteClientCommand> for DeleteClientHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteClient";

    async fn handle(
        &self,
        request: DeleteClientCommand,
        context: &AuthorizedContext,
    ) -> Result<ClientId, fineract_command::CommandError> {
        self.service
            .delete_client(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct ActivateClientHandler<S: ClientService> {
    service: Arc<S>,
}

impl<S: ClientService> ActivateClientHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ClientService> CommandHandler<ActivateClientCommand> for ActivateClientHandler<S> {
    const COMMAND_TYPE: &'static str = "ActivateClient";

    async fn handle(
        &self,
        request: ActivateClientCommand,
        context: &AuthorizedContext,
    ) -> Result<ClientId, fineract_command::CommandError> {
        self.service
            .activate_client(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct CreateGroupHandler<S: ClientService> {
    service: Arc<S>,
}

impl<S: ClientService> CreateGroupHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ClientService> CommandHandler<CreateGroupCommand> for CreateGroupHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateGroup";

    async fn handle(
        &self,
        request: CreateGroupCommand,
        context: &AuthorizedContext,
    ) -> Result<GroupId, fineract_command::CommandError> {
        let mut group = crate::domain::Group::new(
            GroupId::new(1),
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            request.display_name.clone(),
            request.staff_id.map(fineract_core::ids::StaffId::new),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        group.name = request.name;
        group.staff_id = request.staff_id.map(fineract_core::ids::StaffId::new);
        group.external_id = request.external_id;
        group.active = request.active;

        self.service
            .create_group(context, group)
            .await
            .map(|g| g.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateGroupHandler<S: ClientService> {
    service: Arc<S>,
}

impl<S: ClientService> UpdateGroupHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ClientService> CommandHandler<UpdateGroupCommand> for UpdateGroupHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateGroup";

    async fn handle(
        &self,
        request: UpdateGroupCommand,
        context: &AuthorizedContext,
    ) -> Result<GroupId, fineract_command::CommandError> {
        let mut group = crate::domain::Group::new(
            request.id,
            request.tenant_id,
            fineract_core::ids::OfficeId::new(request.office_id),
            request.display_name.clone(),
            request.staff_id.map(fineract_core::ids::StaffId::new),
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        group.name = request.name;
        group.staff_id = request.staff_id.map(fineract_core::ids::StaffId::new);
        group.external_id = request.external_id;
        group.active = request.active;

        self.service
            .update_group(context, group)
            .await
            .map(|g| g.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteGroupHandler<S: ClientService> {
    service: Arc<S>,
}

impl<S: ClientService> DeleteGroupHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: ClientService> CommandHandler<DeleteGroupCommand> for DeleteGroupHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteGroup";

    async fn handle(
        &self,
        request: DeleteGroupCommand,
        context: &AuthorizedContext,
    ) -> Result<GroupId, fineract_command::CommandError> {
        self.service
            .delete_group(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: ClientService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateClientHandler::new(service.clone()));
    dispatcher.register_handler(UpdateClientHandler::new(service.clone()));
    dispatcher.register_handler(DeleteClientHandler::new(service.clone()));
    dispatcher.register_handler(ActivateClientHandler::new(service.clone()));
    dispatcher.register_handler(CreateGroupHandler::new(service.clone()));
    dispatcher.register_handler(UpdateGroupHandler::new(service.clone()));
    dispatcher.register_handler(DeleteGroupHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateClientCommand::COMMAND_TYPE, "CreateClient");
        assert_eq!(UpdateClientCommand::COMMAND_TYPE, "UpdateClient");
        assert_eq!(DeleteClientCommand::COMMAND_TYPE, "DeleteClient");
        assert_eq!(ActivateClientCommand::COMMAND_TYPE, "ActivateClient");
        assert_eq!(CreateGroupCommand::COMMAND_TYPE, "CreateGroup");
        assert_eq!(UpdateGroupCommand::COMMAND_TYPE, "UpdateGroup");
        assert_eq!(DeleteGroupCommand::COMMAND_TYPE, "DeleteGroup");
    }
}
