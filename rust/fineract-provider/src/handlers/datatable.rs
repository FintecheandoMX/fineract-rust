//! Datatable HTTP handlers.
//! Datatable, column, and view management endpoints.

use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_core::ids::{DatatableId, TenantId};
use fineract_datatable::{
    CreateDatatableColumnCommand, CreateDatatableCommand, CreateDatatableViewCommand, Datatable,
    DatatableColumn, DatatableColumnId, DatatableRepository, DatatableView, DatatableViewId,
    DeleteDatatableColumnCommand, DeleteDatatableCommand, DeleteDatatableViewCommand,
    UpdateDatatableColumnCommand, UpdateDatatableCommand, UpdateDatatableViewCommand,
};

#[derive(Debug, Deserialize)]
pub struct CreateDatatableRequest {
    pub registered_table_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub multi_tenant: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDatatableRequest {
    pub registered_table_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub multi_tenant: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumnRequest {
    pub datatable_id: i64,
    pub column_name: String,
    pub display_name: String,
    pub column_type: String,
    pub column_length: Option<i32>,
    pub column_precision: Option<i32>,
    pub column_scale: Option<i32>,
    pub is_mandatory: bool,
    pub is_unique: bool,
    pub is_foreign_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_column: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateColumnRequest {
    pub datatable_id: i64,
    pub column_name: String,
    pub display_name: String,
    pub column_type: String,
    pub column_length: Option<i32>,
    pub column_precision: Option<i32>,
    pub column_scale: Option<i32>,
    pub is_mandatory: bool,
    pub is_unique: bool,
    pub is_foreign_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_column: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateViewRequest {
    pub datatable_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<String>,
    pub filter_criteria: Option<serde_json::Value>,
    pub sort_criteria: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateViewRequest {
    pub datatable_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<String>,
    pub filter_criteria: Option<serde_json::Value>,
    pub sort_criteria: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DatatableResponse {
    pub id: i64,
    pub registered_table_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub multi_tenant: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ColumnResponse {
    pub id: i64,
    pub datatable_id: i64,
    pub column_name: String,
    pub display_name: String,
    pub column_type: String,
    pub column_length: Option<i32>,
    pub column_precision: Option<i32>,
    pub column_scale: Option<i32>,
    pub is_mandatory: bool,
    pub is_unique: bool,
    pub is_foreign_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_column: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ViewResponse {
    pub id: i64,
    pub datatable_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<String>,
    pub filter_criteria: Option<serde_json::Value>,
    pub sort_criteria: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

fn datatable_to_response(dt: Datatable) -> DatatableResponse {
    DatatableResponse {
        id: dt.id.value(),
        registered_table_name: dt.registered_table_name,
        display_name: dt.display_name,
        description: dt.description,
        multi_tenant: dt.multi_tenant,
        created_at: dt.created_at.to_rfc3339(),
        updated_at: dt.updated_at.map(|d| d.to_rfc3339()),
    }
}

fn column_to_response(col: DatatableColumn) -> ColumnResponse {
    ColumnResponse {
        id: col.id.value(),
        datatable_id: col.datatable_id.value(),
        column_name: col.column_name,
        display_name: col.display_name,
        column_type: col.column_type,
        column_length: col.column_length,
        column_precision: col.column_precision,
        column_scale: col.column_scale,
        is_mandatory: col.is_mandatory,
        is_unique: col.is_unique,
        is_foreign_key: col.is_foreign_key,
        foreign_key_table: col.foreign_key_table,
        foreign_key_column: col.foreign_key_column,
        created_at: col.created_at.to_rfc3339(),
        updated_at: col.updated_at.map(|d| d.to_rfc3339()),
    }
}

fn view_to_response(view: DatatableView) -> ViewResponse {
    ViewResponse {
        id: view.id.value(),
        datatable_id: view.datatable_id.value(),
        name: view.name,
        description: view.description,
        columns: view.columns,
        filter_criteria: view.filter_criteria,
        sort_criteria: view.sort_criteria,
        created_at: view.created_at.to_rfc3339(),
        updated_at: view.updated_at.map(|d| d.to_rfc3339()),
    }
}

/// GET /v1/datatables — list all datatables for the tenant.
pub async fn list_datatables(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DatatableResponse>>, AppError> {
    let ctx = &auth.0;
    let datatables = state
        .datatable_repository
        .find_all_datatables(ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        datatables.into_iter().map(datatable_to_response).collect(),
    ))
}

/// GET /v1/datatables/:id — get a single datatable.
pub async fn get_datatable(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<DatatableResponse>, AppError> {
    let ctx = &auth.0;
    let datatable = state
        .datatable_repository
        .find_datatable_by_id(ctx, DatatableId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match datatable {
        Some(dt) => Ok(Json(datatable_to_response(dt))),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/datatables — create a datatable via the command pipeline.
pub async fn create_datatable(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateDatatableRequest>,
) -> Result<Json<DatatableResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        CreateDatatableCommand {
            tenant_id: ctx.tenant_id,
            registered_table_name: payload.registered_table_name,
            display_name: payload.display_name,
            description: payload.description,
            multi_tenant: payload.multi_tenant,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let datatable_id = result.response.ok_or(AppError::Internal)?;
            let datatable = state
                .datatable_repository
                .find_datatable_by_id(ctx, datatable_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(datatable_to_response(datatable)))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/datatables/:id — update a datatable.
pub async fn update_datatable(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateDatatableRequest>,
) -> Result<Json<DatatableResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        UpdateDatatableCommand {
            id: DatatableId::new(id),
            tenant_id: ctx.tenant_id,
            registered_table_name: payload.registered_table_name,
            display_name: payload.display_name,
            description: payload.description,
            multi_tenant: payload.multi_tenant,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let datatable_id = result.response.ok_or(AppError::Internal)?;
            let datatable = state
                .datatable_repository
                .find_datatable_by_id(ctx, datatable_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(datatable_to_response(datatable)))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/datatables/:id — delete a datatable.
pub async fn delete_datatable(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        DeleteDatatableCommand {
            id: DatatableId::new(id),
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

// --- Column Endpoints ---

/// GET /v1/datatables/:datatable_id/columns — list columns for a datatable.
pub async fn list_columns(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(datatable_id): Path<i64>,
) -> Result<Json<Vec<ColumnResponse>>, AppError> {
    let ctx = &auth.0;
    let columns = state
        .datatable_repository
        .find_columns_by_datatable_id(ctx, DatatableId::new(datatable_id))
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(columns.into_iter().map(column_to_response).collect()))
}

/// GET /v1/datatables/:datatable_id/columns/:id — get a single column.
pub async fn get_column(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((datatable_id, id)): Path<(i64, i64)>,
) -> Result<Json<ColumnResponse>, AppError> {
    let ctx = &auth.0;
    let column = state
        .datatable_repository
        .find_column_by_id(ctx, DatatableColumnId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match column {
        Some(c) => {
            if c.datatable_id.value() != datatable_id {
                return Err(AppError::NotFound);
            }
            Ok(Json(column_to_response(c)))
        }
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/datatables/:datatable_id/columns — create a column.
pub async fn create_column(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(_datatable_id): Path<i64>,
    Json(payload): Json<CreateColumnRequest>,
) -> Result<Json<ColumnResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        CreateDatatableColumnCommand {
            tenant_id: ctx.tenant_id,
            datatable_id: DatatableId::new(payload.datatable_id),
            column_name: payload.column_name,
            display_name: payload.display_name,
            column_type: payload.column_type,
            column_length: payload.column_length,
            column_precision: payload.column_precision,
            column_scale: payload.column_scale,
            is_mandatory: payload.is_mandatory,
            is_unique: payload.is_unique,
            is_foreign_key: payload.is_foreign_key,
            foreign_key_table: payload.foreign_key_table,
            foreign_key_column: payload.foreign_key_column,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let column_id = result.response.ok_or(AppError::Internal)?;
            let column = state
                .datatable_repository
                .find_column_by_id(ctx, column_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(column_to_response(column)))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/datatables/:datatable_id/columns/:id — update a column.
pub async fn update_column(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((_datatable_id, id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateColumnRequest>,
) -> Result<Json<ColumnResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        UpdateDatatableColumnCommand {
            id: DatatableColumnId::new(id),
            tenant_id: ctx.tenant_id,
            datatable_id: DatatableId::new(payload.datatable_id),
            column_name: payload.column_name,
            display_name: payload.display_name,
            column_type: payload.column_type,
            column_length: payload.column_length,
            column_precision: payload.column_precision,
            column_scale: payload.column_scale,
            is_mandatory: payload.is_mandatory,
            is_unique: payload.is_unique,
            is_foreign_key: payload.is_foreign_key,
            foreign_key_table: payload.foreign_key_table,
            foreign_key_column: payload.foreign_key_column,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let column_id = result.response.ok_or(AppError::Internal)?;
            let column = state
                .datatable_repository
                .find_column_by_id(ctx, column_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(column_to_response(column)))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/datatables/:datatable_id/columns/:id — delete a column.
pub async fn delete_column(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((_datatable_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        DeleteDatatableColumnCommand {
            id: DatatableColumnId::new(id),
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

// --- View Endpoints ---

/// GET /v1/datatables/:datatable_id/views — list views for a datatable.
pub async fn list_views(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(datatable_id): Path<i64>,
) -> Result<Json<Vec<ViewResponse>>, AppError> {
    let ctx = &auth.0;
    let views = state
        .datatable_repository
        .find_views_by_datatable_id(ctx, DatatableId::new(datatable_id))
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(views.into_iter().map(view_to_response).collect()))
}

/// GET /v1/datatables/:datatable_id/views/:id — get a single view.
pub async fn get_view(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((datatable_id, id)): Path<(i64, i64)>,
) -> Result<Json<ViewResponse>, AppError> {
    let ctx = &auth.0;
    let view = state
        .datatable_repository
        .find_view_by_id(ctx, DatatableViewId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match view {
        Some(v) => {
            if v.datatable_id.value() != datatable_id {
                return Err(AppError::NotFound);
            }
            Ok(Json(view_to_response(v)))
        }
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/datatables/:datatable_id/views — create a view.
pub async fn create_view(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(_datatable_id): Path<i64>,
    Json(payload): Json<CreateViewRequest>,
) -> Result<Json<ViewResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        CreateDatatableViewCommand {
            tenant_id: ctx.tenant_id,
            datatable_id: DatatableId::new(payload.datatable_id),
            name: payload.name,
            description: payload.description,
            columns: payload.columns,
            filter_criteria: payload.filter_criteria,
            sort_criteria: payload.sort_criteria,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let view_id = result.response.ok_or(AppError::Internal)?;
            let view = state
                .datatable_repository
                .find_view_by_id(ctx, view_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(view_to_response(view)))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/datatables/:datatable_id/views/:id — update a view.
pub async fn update_view(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((_datatable_id, id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateViewRequest>,
) -> Result<Json<ViewResponse>, AppError> {
    let ctx = &auth.0;

    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        UpdateDatatableViewCommand {
            id: DatatableViewId::new(id),
            tenant_id: ctx.tenant_id,
            datatable_id: DatatableId::new(payload.datatable_id),
            name: payload.name,
            description: payload.description,
            columns: payload.columns,
            filter_criteria: payload.filter_criteria,
            sort_criteria: payload.sort_criteria,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let view_id = result.response.ok_or(AppError::Internal)?;
            let view = state
                .datatable_repository
                .find_view_by_id(ctx, view_id)
                .await
                .map_err(|_| AppError::Internal)?
                .ok_or(AppError::Internal)?;
            Ok(Json(view_to_response(view)))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/datatables/:datatable_id/views/:id — delete a view.
pub async fn delete_view(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((_datatable_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        DeleteDatatableViewCommand {
            id: DatatableViewId::new(id),
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
