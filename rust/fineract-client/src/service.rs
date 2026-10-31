//! Client domain service with authorization enforcement.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Client, ClientError, ClientType, Group, GroupStatus};
use crate::repository::{ClientRepository, GroupRepository};
use fineract_core::ids::ClientId;
use fineract_core::ids::GroupId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type ClientServiceResult<T> = Result<T, ClientError>;

#[async_trait]
pub trait ClientService: Send + Sync {
    async fn create_client(
        &self,
        context: &AuthorizedContext,
        client: Client,
    ) -> ClientServiceResult<Client>;
    async fn get_client(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientServiceResult<Option<Client>>;
    async fn list_clients(&self, context: &AuthorizedContext) -> ClientServiceResult<Vec<Client>>;
    async fn list_clients_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> ClientServiceResult<Vec<Client>>;
    async fn update_client(
        &self,
        context: &AuthorizedContext,
        client: Client,
    ) -> ClientServiceResult<Client>;
    async fn delete_client(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientServiceResult<ClientId>;
    async fn activate_client(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientServiceResult<ClientId>;
    // Group operations
    async fn create_group(
        &self,
        context: &AuthorizedContext,
        group: crate::domain::Group,
    ) -> ClientServiceResult<crate::domain::Group>;
    async fn get_group(
        &self,
        context: &AuthorizedContext,
        id: GroupId,
    ) -> ClientServiceResult<Option<crate::domain::Group>>;
    async fn list_groups(
        &self,
        context: &AuthorizedContext,
    ) -> ClientServiceResult<Vec<crate::domain::Group>>;
    async fn update_group(
        &self,
        context: &AuthorizedContext,
        group: crate::domain::Group,
    ) -> ClientServiceResult<crate::domain::Group>;
    async fn delete_group(
        &self,
        context: &AuthorizedContext,
        id: GroupId,
    ) -> ClientServiceResult<GroupId>;
}

pub struct ClientServiceImpl<R: ClientRepository, G: GroupRepository> {
    client_repository: R,
    group_repository: G,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: ClientRepository, G: GroupRepository> ClientServiceImpl<R, G> {
    pub fn new(
        client_repository: R,
        group_repository: G,
        auth_service: Arc<dyn AuthorizationService>,
    ) -> Self {
        Self {
            client_repository,
            group_repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: ClientRepository, G: GroupRepository> ClientService for ClientServiceImpl<R, G> {
    async fn create_client(
        &self,
        context: &AuthorizedContext,
        client: Client,
    ) -> ClientServiceResult<Client> {
        self.auth_service
            .check(context, Permission::create_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        if client.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        self.client_repository.insert(context, &client).await?;
        Ok(client)
    }

    async fn get_client(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientServiceResult<Option<Client>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        let client = self.client_repository.find_by_id(context, id).await?;

        if let Some(ref c) = client {
            if c.tenant_id != context.tenant_id {
                return Err(ClientError::TenantIsolationViolation);
            }
        }

        Ok(client)
    }

    async fn list_clients(&self, context: &AuthorizedContext) -> ClientServiceResult<Vec<Client>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        self.client_repository.find_all_by_tenant(context).await
    }

    async fn list_clients_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> ClientServiceResult<Vec<Client>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        self.client_repository
            .find_all_by_office(context, office_id)
            .await
    }

    async fn update_client(
        &self,
        context: &AuthorizedContext,
        client: Client,
    ) -> ClientServiceResult<Client> {
        self.auth_service
            .check(context, Permission::update_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        if client.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        let existing = self
            .client_repository
            .find_by_id(context, client.id)
            .await?
            .ok_or(ClientError::NotFound(client.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        self.client_repository.update(context, &client).await?;
        Ok(client)
    }

    async fn delete_client(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientServiceResult<ClientId> {
        self.auth_service
            .check(context, Permission::update_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        let client = self
            .client_repository
            .find_by_id(context, id)
            .await?
            .ok_or(ClientError::NotFound(id))?;

        if client.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        self.client_repository.delete(context, id).await?;
        Ok(id)
    }

    async fn activate_client(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientServiceResult<ClientId> {
        self.auth_service
            .check(context, Permission::update_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        let mut client = self
            .client_repository
            .find_by_id(context, id)
            .await?
            .ok_or(ClientError::NotFound(id))?;

        if client.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        client.status = crate::domain::ClientStatus::Active;
        client.active = true;
        self.client_repository.update(context, &client).await?;
        Ok(id)
    }

    async fn create_group(
        &self,
        context: &AuthorizedContext,
        group: crate::domain::Group,
    ) -> ClientServiceResult<crate::domain::Group> {
        self.auth_service
            .check(context, Permission::create_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        if group.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        self.group_repository.insert(context, &group).await?;
        Ok(group)
    }

    async fn get_group(
        &self,
        context: &AuthorizedContext,
        id: GroupId,
    ) -> ClientServiceResult<Option<crate::domain::Group>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        let group = self.group_repository.find_by_id(context, id).await?;

        if let Some(ref g) = group {
            if g.tenant_id != context.tenant_id {
                return Err(ClientError::TenantIsolationViolation);
            }
        }

        Ok(group)
    }

    async fn list_groups(
        &self,
        context: &AuthorizedContext,
    ) -> ClientServiceResult<Vec<crate::domain::Group>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        self.group_repository.find_all_by_tenant(context).await
    }

    async fn update_group(
        &self,
        context: &AuthorizedContext,
        group: crate::domain::Group,
    ) -> ClientServiceResult<crate::domain::Group> {
        self.auth_service
            .check(context, Permission::update_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        if group.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        let existing = self
            .group_repository
            .find_by_id(context, group.id)
            .await?
            .ok_or(ClientError::GroupNotFound(group.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        self.group_repository.update(context, &group).await?;
        Ok(group)
    }

    async fn delete_group(
        &self,
        context: &AuthorizedContext,
        id: GroupId,
    ) -> ClientServiceResult<GroupId> {
        self.auth_service
            .check(context, Permission::update_client())
            .await
            .map_err(|e| ClientError::Internal {
                detail: e.to_string(),
            })?;

        let group = self
            .group_repository
            .find_by_id(context, id)
            .await?
            .ok_or(ClientError::GroupNotFound(id))?;

        if group.tenant_id != context.tenant_id {
            return Err(ClientError::TenantIsolationViolation);
        }

        self.group_repository.delete(context, id).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn ClientService) {}
    }
}
