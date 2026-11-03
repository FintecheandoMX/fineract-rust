//! Note HTTP handlers.
//! Note management for clients, loans, savings, groups, centers.

use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::NoteId;
use fineract_core::ids::TenantId;
use fineract_note::CreateNoteCommand;
use fineract_note::DeleteNoteCommand;
use fineract_note::Note;
use fineract_note::NoteEntityType;
use fineract_note::NoteRepository;
use fineract_note::UpdateNoteCommand;

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub entity_type: String,
    pub entity_id: i64,
    pub note: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub entity_type: String,
    pub entity_id: i64,
    pub note: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct NoteQueryParams {
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct NoteResponse {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub note: String,
    pub created_by: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub active: bool,
}

fn entity_type_to_string(et: NoteEntityType) -> String {
    match et {
        NoteEntityType::Client => "client".to_string(),
        NoteEntityType::Loan => "loan".to_string(),
        NoteEntityType::Savings => "savings".to_string(),
        NoteEntityType::Group => "group".to_string(),
        NoteEntityType::Center => "center".to_string(),
    }
}

/// GET /v1/notes — list notes for the tenant, optionally filtered by entity.
pub async fn list_notes(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<NoteQueryParams>,
) -> Result<Json<Vec<NoteResponse>>, AppError> {
    let ctx = &auth.0;

    let notes =
        if let (Some(entity_type_str), Some(entity_id)) = (params.entity_type, params.entity_id) {
            let entity_type = match entity_type_str.to_lowercase().as_str() {
                "client" => NoteEntityType::Client,
                "loan" => NoteEntityType::Loan,
                "savings" => NoteEntityType::Savings,
                "group" => NoteEntityType::Group,
                "center" => NoteEntityType::Center,
                _ => return Err(AppError::BadRequest("invalid entity_type".to_string())),
            };
            state
                .note_repository
                .find_by_entity(ctx, entity_type, entity_id)
                .await
                .map_err(|_| AppError::Internal)?
        } else {
            state
                .note_repository
                .find_all_by_tenant(ctx)
                .await
                .map_err(|_| AppError::Internal)?
        };

    Ok(Json(
        notes
            .into_iter()
            .map(|n| NoteResponse {
                id: n.id.value(),
                entity_type: entity_type_to_string(n.entity_type),
                entity_id: n.entity_id,
                note: n.note,
                created_by: n.created_by,
                created_at: n.created_at.to_string(),
                updated_at: n.updated_at.map(|dt| dt.to_string()),
                active: n.active,
            })
            .collect(),
    ))
}

/// GET /v1/notes/:id — get a single note.
pub async fn get_note(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<NoteResponse>, AppError> {
    let ctx = &auth.0;
    let note = state
        .note_repository
        .find_by_id(ctx, NoteId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match note {
        Some(n) => Ok(Json(NoteResponse {
            id: n.id.value(),
            entity_type: entity_type_to_string(n.entity_type),
            entity_id: n.entity_id,
            note: n.note,
            created_by: n.created_by,
            created_at: n.created_at.to_string(),
            updated_at: n.updated_at.map(|dt| dt.to_string()),
            active: n.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/notes — create a note via the command pipeline.
pub async fn create_note(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<Json<NoteResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_note::CreateNoteCommand {
            tenant_id: ctx.tenant_id,
            entity_type: payload.entity_type,
            entity_id: payload.entity_id,
            note: payload.note,
            created_by: ctx.principal.app_user_id.value(),
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let note_id = result.response.ok_or(AppError::Internal)?;
            let note = state
                .note_repository
                .find_by_id(ctx, note_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(NoteResponse {
                id: note.id.value(),
                entity_type: entity_type_to_string(note.entity_type),
                entity_id: note.entity_id,
                note: note.note,
                created_by: note.created_by,
                created_at: note.created_at.to_string(),
                updated_at: note.updated_at.map(|dt| dt.to_string()),
                active: note.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/notes/:id — update a note.
pub async fn update_note(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<NoteResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_note::UpdateNoteCommand {
            id: NoteId::new(id),
            tenant_id: ctx.tenant_id,
            entity_type: payload.entity_type,
            entity_id: payload.entity_id,
            note: payload.note,
            created_by: ctx.principal.app_user_id.value(),
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
            let note_id = result.response.ok_or(AppError::Internal)?;
            let note = state
                .note_repository
                .find_by_id(ctx, note_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(NoteResponse {
                id: note.id.value(),
                entity_type: entity_type_to_string(note.entity_type),
                entity_id: note.entity_id,
                note: note.note,
                created_by: note.created_by,
                created_at: note.created_at.to_string(),
                updated_at: note.updated_at.map(|dt| dt.to_string()),
                active: note.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/notes/:id — delete a note.
pub async fn delete_note(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_note::DeleteNoteCommand {
            id: NoteId::new(id),
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
