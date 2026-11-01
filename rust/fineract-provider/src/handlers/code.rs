//! Code and CodeValue HTTP handlers.
//! Lookup table management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_code::Code;
use fineract_code::CodeRepository;
use fineract_code::CodeValue;
use fineract_code::CreateCodeCommand;
use fineract_code::CreateCodeValueCommand;
use fineract_code::DeleteCodeCommand;
use fineract_code::DeleteCodeValueCommand;
use fineract_code::UpdateCodeCommand;
use fineract_code::UpdateCodeValueCommand;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::CodeId;
use fineract_core::ids::CodeValueId;
use fineract_core::ids::TenantId;

#[derive(Debug, Deserialize)]
pub struct CreateCodeRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCodeRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateCodeValueRequest {
    pub code_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub position: i32,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCodeValueRequest {
    pub name: String,
    pub description: Option<String>,
    pub position: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct CodeResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct CodeValueResponse {
    pub id: i64,
    pub code_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub position: i32,
    pub is_active: bool,
}

/// GET /v1/codes — list codes for the tenant.
pub async fn list_codes(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<CodeResponse>>, AppError> {
    let ctx = &auth.0;
    let codes = state
        .code_repository
        .find_all_codes(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        codes
            .into_iter()
            .map(|c| CodeResponse {
                id: c.id.value(),
                name: c.name,
                description: c.description,
                is_active: c.is_active,
            })
            .collect(),
    ))
}

/// GET /v1/codes/:id — get a single code.
pub async fn get_code(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CodeResponse>, AppError> {
    let ctx = &auth.0;
    let code = state
        .code_repository
        .find_code_by_id(ctx, CodeId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match code {
        Some(c) => Ok(Json(CodeResponse {
            id: c.id.value(),
            name: c.name,
            description: c.description,
            is_active: c.is_active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// GET /v1/codes/:codeId/codeValues — get code values for a code.
pub async fn list_code_values(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(code_id): Path<i64>,
) -> Result<Json<Vec<CodeValueResponse>>, AppError> {
    let ctx = &auth.0;
    let code_values = state
        .code_repository
        .find_code_values_by_code_id(ctx, CodeId::new(code_id))
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        code_values
            .into_iter()
            .map(|cv| CodeValueResponse {
                id: cv.id.value(),
                code_id: cv.code_id.value(),
                name: cv.name,
                description: cv.description,
                position: cv.position,
                is_active: cv.is_active,
            })
            .collect(),
    ))
}

/// GET /v1/codes/:codeId/codeValues/:id — get a single code value.
pub async fn get_code_value(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((code_id, id)): Path<(i64, i64)>,
) -> Result<Json<CodeValueResponse>, AppError> {
    let ctx = &auth.0;
    let cv = state
        .code_repository
        .find_code_value_by_id(ctx, CodeValueId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match cv {
        Some(c) => {
            if c.code_id.value() != code_id {
                return Err(AppError::NotFound);
            }
            Ok(Json(CodeValueResponse {
                id: c.id.value(),
                code_id: c.code_id.value(),
                name: c.name,
                description: c.description,
                position: c.position,
                is_active: c.is_active,
            }))
        }
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/codes — create a code via the command pipeline.
pub async fn create_code(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateCodeRequest>,
) -> Result<Json<CodeResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_code::CreateCodeCommand {
            tenant_id: ctx.tenant_id,
            name: payload.name,
            description: payload.description,
            is_active: payload.is_active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let code_id = result.response.ok_or(AppError::Internal)?;
            let code = state
                .code_repository
                .find_code_by_id(ctx, code_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CodeResponse {
                id: code.id.value(),
                name: code.name,
                description: code.description,
                is_active: code.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/codes/:id — update a code.
pub async fn update_code(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCodeRequest>,
) -> Result<Json<CodeResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_code::UpdateCodeCommand {
            id: CodeId::new(id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            description: payload.description,
            is_active: payload.is_active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let code_id = result.response.ok_or(AppError::Internal)?;
            let code = state
                .code_repository
                .find_code_by_id(ctx, code_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CodeResponse {
                id: code.id.value(),
                name: code.name,
                description: code.description,
                is_active: code.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/codes/:id — delete a code.
pub async fn delete_code(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_code::DeleteCodeCommand {
            id: CodeId::new(id),
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

/// POST /v1/codes/:codeId/codeValues — create a code value via the command pipeline.
pub async fn create_code_value(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(code_id): Path<i64>,
    Json(payload): Json<CreateCodeValueRequest>,
) -> Result<Json<CodeValueResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_code::CreateCodeValueCommand {
            code_id: CodeId::new(code_id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            description: payload.description,
            position: payload.position,
            is_active: payload.is_active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let cv_id = result.response.ok_or(AppError::Internal)?;
            let cv = state
                .code_repository
                .find_code_value_by_id(ctx, cv_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CodeValueResponse {
                id: cv.id.value(),
                code_id: cv.code_id.value(),
                name: cv.name,
                description: cv.description,
                position: cv.position,
                is_active: cv.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/codes/:codeId/codeValues/:id — update a code value.
pub async fn update_code_value(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((code_id, id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateCodeValueRequest>,
) -> Result<Json<CodeValueResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_code::UpdateCodeValueCommand {
            id: CodeValueId::new(id),
            code_id: CodeId::new(code_id),
            tenant_id: ctx.tenant_id,
            name: payload.name,
            description: payload.description,
            position: payload.position,
            is_active: payload.is_active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let cv_id = result.response.ok_or(AppError::Internal)?;
            let cv = state
                .code_repository
                .find_code_value_by_id(ctx, cv_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(CodeValueResponse {
                id: cv.id.value(),
                code_id: cv.code_id.value(),
                name: cv.name,
                description: cv.description,
                position: cv.position,
                is_active: cv.is_active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/codes/:codeId/codeValues/:id — delete a code value.
pub async fn delete_code_value(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((_code_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_code::DeleteCodeValueCommand {
            id: CodeValueId::new(id),
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
