//! Command handlers for note operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Note;
use crate::domain::NoteEntityType;
use crate::service::NoteService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::NoteId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteCommand {
    pub tenant_id: TenantId,
    pub entity_type: String,
    pub entity_id: i64,
    pub note: String,
    pub created_by: i64,
}

impl CommandRequest for CreateNoteCommand {
    type Response = NoteId;
    const COMMAND_TYPE: &'static str = "CreateNote";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNoteCommand {
    pub id: NoteId,
    pub tenant_id: TenantId,
    pub entity_type: String,
    pub entity_id: i64,
    pub note: String,
    pub created_by: i64,
    pub active: bool,
}

impl CommandRequest for UpdateNoteCommand {
    type Response = NoteId;
    const COMMAND_TYPE: &'static str = "UpdateNote";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNoteCommand {
    pub id: NoteId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteNoteCommand {
    type Response = NoteId;
    const COMMAND_TYPE: &'static str = "DeleteNote";
}

fn parse_entity_type(s: &str) -> Result<NoteEntityType, String> {
    match s.to_lowercase().as_str() {
        "client" => Ok(NoteEntityType::Client),
        "loan" => Ok(NoteEntityType::Loan),
        "savings" => Ok(NoteEntityType::Savings),
        "group" => Ok(NoteEntityType::Group),
        "center" => Ok(NoteEntityType::Center),
        _ => Err(format!("invalid entity_type: {s}")),
    }
}

pub struct CreateNoteHandler<S: NoteService> {
    service: Arc<S>,
}

impl<S: NoteService> CreateNoteHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: NoteService> CommandHandler<CreateNoteCommand> for CreateNoteHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateNote";

    async fn handle(
        &self,
        request: CreateNoteCommand,
        context: &AuthorizedContext,
    ) -> Result<NoteId, fineract_command::CommandError> {
        let entity_type = parse_entity_type(&request.entity_type)
            .map_err(fineract_command::CommandError::HandlerError)?;

        let note = Note::new(
            NoteId::new(1),
            request.tenant_id,
            entity_type,
            request.entity_id,
            request.note,
            request.created_by,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_note(context, note)
            .await
            .map(|n| n.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateNoteHandler<S: NoteService> {
    service: Arc<S>,
}

impl<S: NoteService> UpdateNoteHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: NoteService> CommandHandler<UpdateNoteCommand> for UpdateNoteHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateNote";

    async fn handle(
        &self,
        request: UpdateNoteCommand,
        context: &AuthorizedContext,
    ) -> Result<NoteId, fineract_command::CommandError> {
        let entity_type = parse_entity_type(&request.entity_type)
            .map_err(fineract_command::CommandError::HandlerError)?;

        let note = Note::new(
            request.id,
            request.tenant_id,
            entity_type,
            request.entity_id,
            request.note,
            request.created_by,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut note = note;
        note.active = request.active;

        self.service
            .update_note(context, note)
            .await
            .map(|n| n.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteNoteHandler<S: NoteService> {
    service: Arc<S>,
}

impl<S: NoteService> DeleteNoteHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: NoteService> CommandHandler<DeleteNoteCommand> for DeleteNoteHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteNote";

    async fn handle(
        &self,
        request: DeleteNoteCommand,
        context: &AuthorizedContext,
    ) -> Result<NoteId, fineract_command::CommandError> {
        self.service
            .delete_note(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: NoteService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateNoteHandler::new(service.clone()));
    dispatcher.register_handler(UpdateNoteHandler::new(service.clone()));
    dispatcher.register_handler(DeleteNoteHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateNoteCommand::COMMAND_TYPE, "CreateNote");
        assert_eq!(UpdateNoteCommand::COMMAND_TYPE, "UpdateNote");
        assert_eq!(DeleteNoteCommand::COMMAND_TYPE, "DeleteNote");
    }
}
