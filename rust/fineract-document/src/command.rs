//! Command handlers for document operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Document;
use crate::service::DocumentService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::DocumentId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub content_type: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub active: bool,
}

impl CommandRequest for CreateDocumentCommand {
    type Response = DocumentId;
    const COMMAND_TYPE: &'static str = "CreateDocument";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentCommand {
    pub id: DocumentId,
    pub tenant_id: TenantId,
    pub name: String,
    pub content_type: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub active: bool,
}

impl CommandRequest for UpdateDocumentCommand {
    type Response = DocumentId;
    const COMMAND_TYPE: &'static str = "UpdateDocument";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDocumentCommand {
    pub id: DocumentId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteDocumentCommand {
    type Response = DocumentId;
    const COMMAND_TYPE: &'static str = "DeleteDocument";
}

pub struct CreateDocumentHandler<S: DocumentService> {
    service: Arc<S>,
}

impl<S: DocumentService> CreateDocumentHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DocumentService> CommandHandler<CreateDocumentCommand> for CreateDocumentHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateDocument";

    async fn handle(
        &self,
        request: CreateDocumentCommand,
        context: &AuthorizedContext,
    ) -> Result<DocumentId, fineract_command::CommandError> {
        let doc = Document::new(
            DocumentId::new(1),
            request.tenant_id,
            request.name,
            request.content_type,
            request.storage_path,
            request.size_bytes,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .create_document(context, doc)
            .await
            .map(|d| d.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateDocumentHandler<S: DocumentService> {
    service: Arc<S>,
}

impl<S: DocumentService> UpdateDocumentHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DocumentService> CommandHandler<UpdateDocumentCommand> for UpdateDocumentHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateDocument";

    async fn handle(
        &self,
        request: UpdateDocumentCommand,
        context: &AuthorizedContext,
    ) -> Result<DocumentId, fineract_command::CommandError> {
        let doc = Document::new(
            request.id,
            request.tenant_id,
            request.name,
            request.content_type,
            request.storage_path,
            request.size_bytes,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;
        self.service
            .update_document(context, doc)
            .await
            .map(|d| d.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteDocumentHandler<S: DocumentService> {
    service: Arc<S>,
}

impl<S: DocumentService> DeleteDocumentHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: DocumentService> CommandHandler<DeleteDocumentCommand> for DeleteDocumentHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteDocument";

    async fn handle(
        &self,
        request: DeleteDocumentCommand,
        context: &AuthorizedContext,
    ) -> Result<DocumentId, fineract_command::CommandError> {
        self.service
            .delete_document(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: DocumentService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateDocumentHandler::new(service.clone()));
    dispatcher.register_handler(UpdateDocumentHandler::new(service.clone()));
    dispatcher.register_handler(DeleteDocumentHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateDocumentCommand::COMMAND_TYPE, "CreateDocument");
        assert_eq!(UpdateDocumentCommand::COMMAND_TYPE, "UpdateDocument");
        assert_eq!(DeleteDocumentCommand::COMMAND_TYPE, "DeleteDocument");
    }
}
