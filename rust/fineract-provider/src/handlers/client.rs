//! Client HTTP handlers.
//! Clients and groups management.

use axum::extract::{Extension, Path, State};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::error::AppError;
use crate::state::AppState;
use chrono::Utc;
use fineract_client::ActivateClientCommand;
use fineract_client::Client;
use fineract_client::ClientRepository;
use fineract_client::ClientStatus;
use fineract_client::ClientType;
use fineract_client::CreateClientCommand;
use fineract_client::CreateGroupCommand;
use fineract_client::DeleteClientCommand;
use fineract_client::DeleteGroupCommand;
use fineract_client::Group;
use fineract_client::GroupRepository;
use fineract_client::GroupStatus;
use fineract_client::UpdateClientCommand;
use fineract_client::UpdateGroupCommand;
use fineract_command::Command;
use fineract_command::CommandDispatcher;
use fineract_command::CommandRequest;
use fineract_core::ids::ClientId;
use fineract_core::ids::GroupId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

#[derive(Debug, Deserialize)]
pub struct CreateClientRequest {
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

#[derive(Debug, Deserialize)]
pub struct UpdateClientRequest {
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

#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: i64,
    pub display_name: String,
    pub client_type: String,
    pub status: String,
    pub active: bool,
}

/// GET /v1/clients — list clients for the tenant.
pub async fn list_clients(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<ClientResponse>>, AppError> {
    let ctx = &auth.0;
    let clients = ClientRepository::find_all_by_tenant(&*state.client_repository, ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        clients
            .into_iter()
            .map(|c| ClientResponse {
                id: c.id.value(),
                display_name: c.display_name,
                client_type: format!("{:?}", c.client_type),
                status: format!("{:?}", c.status),
                active: c.active,
            })
            .collect(),
    ))
}

/// GET /v1/clients/:id — get a single client.
pub async fn get_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ClientResponse>, AppError> {
    let ctx = &auth.0;
    let client = ClientRepository::find_by_id(&*state.client_repository, ctx, ClientId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match client {
        Some(c) => Ok(Json(ClientResponse {
            id: c.id.value(),
            display_name: c.display_name,
            client_type: format!("{:?}", c.client_type),
            status: format!("{:?}", c.status),
            active: c.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/clients — create a client via the command pipeline.
pub async fn create_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateClientRequest>,
) -> Result<Json<ClientResponse>, AppError> {
    let ctx = &auth.0;
    let client_type = match payload.client_type.as_str() {
        "Individual" => ClientType::Individual,
        "Group" => ClientType::Group,
        _ => return Err(AppError::BadRequest("invalid client type".to_string())),
    };

    let display_name = payload.display_name.clone();
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_client::CreateClientCommand {
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            client_type: format!("{:?}", client_type),
            display_name: display_name.clone(),
            firstname: payload.firstname,
            lastname: payload.lastname,
            date_of_birth: payload.date_of_birth,
            gender: payload.gender,
            mobile_no: payload.mobile_no,
            email: payload.email,
            staff_id: payload.staff_id,
            external_id: payload.external_id,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    match result.state {
        fineract_command::CommandState::Executed => {
            let client_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(ClientResponse {
                id: client_id.value(),
                display_name,
                client_type: payload.client_type,
                status: "Pending".to_string(),
                active: false,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/clients/:id — update a client.
pub async fn update_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateClientRequest>,
) -> Result<Json<ClientResponse>, AppError> {
    let ctx = &auth.0;
    let client_type = match payload.client_type.as_str() {
        "Individual" => fineract_client::ClientType::Individual,
        "Group" => fineract_client::ClientType::Group,
        _ => return Err(AppError::BadRequest("invalid client type".to_string())),
    };

    let display_name = payload.display_name.clone();
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_client::UpdateClientCommand {
            id: ClientId::new(id),
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            client_type: format!("{:?}", client_type),
            display_name: display_name.clone(),
            firstname: payload.firstname,
            lastname: payload.lastname,
            date_of_birth: payload.date_of_birth,
            gender: payload.gender,
            mobile_no: payload.mobile_no,
            email: payload.email,
            staff_id: payload.staff_id,
            external_id: payload.external_id,
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
            let client_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(ClientResponse {
                id: client_id.value(),
                display_name,
                client_type: format!("{:?}", client_type),
                status: "Pending".to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/clients/:id — delete a client.
pub async fn delete_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_client::DeleteClientCommand {
            id: ClientId::new(id),
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

/// POST /v1/clients/:id/activate — activate a client.
pub async fn activate_client(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_client::ActivateClientCommand {
            id: ClientId::new(id),
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

// Group handlers
#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub office_id: i64,
    pub name: String,
    pub display_name: String,
    pub staff_id: Option<i64>,
    pub external_id: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub office_id: i64,
    pub name: String,
    pub display_name: String,
    pub staff_id: Option<i64>,
    pub external_id: Option<String>,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub active: bool,
}

/// GET /v1/groups — list groups for the tenant.
pub async fn list_groups(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
) -> Result<Json<Vec<GroupResponse>>, AppError> {
    let ctx = &auth.0;
    let groups = GroupRepository::find_all_by_tenant(&*state.client_repository, ctx)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(Json(
        groups
            .into_iter()
            .map(|g| GroupResponse {
                id: g.id.value(),
                name: g.name,
                display_name: g.display_name,
                status: format!("{:?}", g.status),
                active: g.active,
            })
            .collect(),
    ))
}

/// GET /v1/groups/:id — get a single group.
pub async fn get_group(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<GroupResponse>, AppError> {
    let ctx = &auth.0;
    let group = GroupRepository::find_by_id(&*state.client_repository, ctx, GroupId::new(id))
        .await
        .map_err(|_| AppError::Internal)?;
    match group {
        Some(g) => Ok(Json(GroupResponse {
            id: g.id.value(),
            name: g.name,
            display_name: g.display_name,
            status: format!("{:?}", g.status),
            active: g.active,
        })),
        None => Err(AppError::NotFound),
    }
}

/// POST /v1/groups — create a group via the command pipeline.
pub async fn create_group(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    let ctx = &auth.0;
    let name = payload.name.clone();
    let display_name = payload.display_name.clone();
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_client::CreateGroupCommand {
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            name: name.clone(),
            display_name: display_name.clone(),
            staff_id: payload.staff_id,
            external_id: payload.external_id,
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
            let group_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(GroupResponse {
                id: group_id.value(),
                name,
                display_name,
                status: "Active".to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// PUT /v1/groups/:id — update a group.
pub async fn update_group(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_client::UpdateGroupCommand {
            id: GroupId::new(id),
            tenant_id: ctx.tenant_id,
            office_id: payload.office_id,
            name: payload.name.clone(),
            display_name: payload.display_name.clone(),
            staff_id: payload.staff_id,
            external_id: payload.external_id,
            active: payload.active,
        },
    );

    let result = state
        .dispatcher
        .dispatch(command, ctx)
        .await
        .map_err(|_| AppError::Internal)?;

    let display_name = payload.display_name.clone();
    match result.state {
        fineract_command::CommandState::Executed => {
            let group_id = result.response.ok_or(AppError::Internal)?;
            Ok(Json(GroupResponse {
                id: group_id.value(),
                name: payload.name,
                display_name,
                status: "Active".to_string(),
                active: payload.active,
            }))
        }
        _ => Err(AppError::Internal),
    }
}

/// DELETE /v1/groups/:id — delete a group.
pub async fn delete_group(
    Extension(auth): Extension<AuthContext>,
    state: State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let ctx = &auth.0;
    let command = Command::new(
        None,
        ctx.tenant_id,
        Some("api".to_string()),
        fineract_client::DeleteGroupCommand {
            id: GroupId::new(id),
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
