//! Document HTTP handlers.
//! Document management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::DocumentId;
use fineract_core::ids::TenantId;
use fineract_document::CreateDocumentCommand;
use fineract_document::DeleteDocumentCommand;
use fineract_document::Document;
use fineract_document::DocumentRepository;
use fineract_document::UpdateDocumentCommand;

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub name: String,
    pub content_type: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub name: String,
    pub content_type: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: i64,
    pub name: String,
    pub content_type: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub active: bool,
}

/// GET /v1/documents — list documents for the tenant.
pub async fn list_documents(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<DocumentResponse>>, AppError> {
    let ctx = &auth.0;
    let docs = state
        .document_repository
        .find_all_by_tenant(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        docs.into_iter()
            .map(|d| DocumentResponse {
                id: d.id.value(),
                name: d.name,
                content_type: d.content_type,
                storage_path: d.storage_path,
                size_bytes: d.size_bytes,
                active: d.active,
            })
            .collect(),
    ))
}

/// GET /v1/documents/:id — get a single document.
pub async fn get_document(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<DocumentResponse>, AppError> {
    let ctx = &auth.0;
    let doc = state
        .document_repository
        .find_by_id(ctx, DocumentId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match doc {
        Some(d) => Ok(Json(DocumentResponse {
            id: d.id.value(),
            name: d.name,
            content_type: d.content_type,
            storage_path: d.storage_path,
            size_bytes: d.size_bytes,
            active: d.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/documents — create a document via the command pipeline.
pub async fn create_document(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, AppError> {
    let ctx = &auth.0;
    let name = payload.name.clone();
    let content_type = payload.content_type.clone();
    let storage_path = payload.storage_path.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_document::CreateDocumentCommand {
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            content_type: content_type.clone(),
            storage_path: storage_path.clone(),
            size_bytes: payload.size_bytes,
            active: payload.active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let doc_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(DocumentResponse {
                id: doc_id.value(),
                name,
                content_type,
                storage_path,
                size_bytes: payload.size_bytes,
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/documents/:id — update a document.
pub async fn update_document(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentResponse>, AppError> {
    let ctx = &auth.0;
    let name = payload.name.clone();
    let content_type = payload.content_type.clone();
    let storage_path = payload.storage_path.clone();

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_document::UpdateDocumentCommand {
            id: DocumentId::new(id),
            tenant_id: ctx.tenant_id,
            name: name.clone(),
            content_type: content_type.clone(),
            storage_path: storage_path.clone(),
            size_bytes: payload.size_bytes,
            active: payload.active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let doc_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(DocumentResponse {
                id: doc_id.value(),
                name,
                content_type,
                storage_path,
                size_bytes: payload.size_bytes,
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/documents/:id — delete a document.
pub async fn delete_document(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_document::DeleteDocumentCommand {
            id: DocumentId::new(id),
            tenant_id: ctx.tenant_id,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => Ok(StatusCode::NO_CONTENT),
        _ => Err(AppError::Internal),
    }
}
