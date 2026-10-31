//! Client repositories with tenant isolation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::domain::{Client, ClientError, ClientStatus, ClientType, Group, GroupStatus};
use fineract_core::ids::ClientId;
use fineract_core::ids::GroupId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type ClientRepositoryResult<T> = Result<T, ClientError>;

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientRepositoryResult<Option<Client>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ClientRepositoryResult<Vec<Client>>;
    async fn find_all_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> ClientRepositoryResult<Vec<Client>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        client: &Client,
    ) -> ClientRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        client: &Client,
    ) -> ClientRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: ClientId)
        -> ClientRepositoryResult<()>;
}

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: GroupId,
    ) -> ClientRepositoryResult<Option<crate::domain::Group>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ClientRepositoryResult<Vec<crate::domain::Group>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        group: &crate::domain::Group,
    ) -> ClientRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        group: &crate::domain::Group,
    ) -> ClientRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: GroupId) -> ClientRepositoryResult<()>;
}

pub struct SqlxClientRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxClientRepository {
    pub fn new(
        pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
        connection_string: String,
    ) -> Self {
        Self {
            pool_manager,
            connection_string,
        }
    }

    async fn pool_for_context(
        &self,
        context: &AuthorizedContext,
    ) -> ClientRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|_| ClientError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl ClientRepository for SqlxClientRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientRepositoryResult<Option<Client>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, office_id, client_type, status, firstname, lastname, display_name, date_of_birth, gender, mobile_no, email, office_name, staff_id, staff_name, external_id, active, created_date, last_modified_date FROM clients WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_client(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ClientRepositoryResult<Vec<Client>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, client_type, status, firstname, lastname, display_name, date_of_birth, gender, mobile_no, email, office_name, staff_id, staff_name, external_id, active, created_date, last_modified_date FROM clients WHERE tenant_id = $1 ORDER BY display_name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_client(&r)).collect()
    }

    async fn find_all_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> ClientRepositoryResult<Vec<Client>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, client_type, status, firstname, lastname, display_name, date_of_birth, gender, mobile_no, email, office_name, staff_id, staff_name, external_id, active, created_date, last_modified_date FROM clients WHERE tenant_id = $1 AND office_id = $2 ORDER BY display_name",
        )
        .bind(context.tenant_id.value())
        .bind(office_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_client(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        client: &Client,
    ) -> ClientRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO clients (id, tenant_id, office_id, client_type, status, firstname, lastname, display_name, date_of_birth, gender, mobile_no, email, office_name, staff_id, staff_name, external_id, active, created_date, last_modified_date) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)",
        )
        .bind(client.id.value())
        .bind(client.tenant_id.value())
        .bind(client.office_id.value())
        .bind(format!("{:?}", client.client_type))
        .bind(format!("{:?}", client.status))
        .bind(&client.firstname)
        .bind(&client.lastname)
        .bind(&client.display_name)
        .bind(client.date_of_birth)
        .bind(&client.gender)
        .bind(&client.mobile_no)
        .bind(&client.email)
        .bind(&client.office_name)
        .bind(client.staff_id.map(|id| id.value()))
        .bind(&client.staff_name)
        .bind(&client.external_id)
        .bind(client.active)
        .bind(client.created_date)
        .bind(client.last_modified_date)
        .execute(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        client: &Client,
    ) -> ClientRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE clients SET office_id = $2, client_type = $3, status = $4, firstname = $5, lastname = $6, display_name = $7, date_of_birth = $8, gender = $9, mobile_no = $10, email = $11, office_name = $12, staff_id = $13, staff_name = $14, external_id = $15, active = $16, last_modified_date = $17 WHERE id = $1 AND tenant_id = $18",
        )
        .bind(client.id.value())
        .bind(client.office_id.value())
        .bind(format!("{:?}", client.client_type))
        .bind(format!("{:?}", client.status))
        .bind(&client.firstname)
        .bind(&client.lastname)
        .bind(&client.display_name)
        .bind(client.date_of_birth)
        .bind(&client.gender)
        .bind(&client.mobile_no)
        .bind(&client.email)
        .bind(&client.office_name)
        .bind(client.staff_id.map(|id| id.value()))
        .bind(&client.staff_name)
        .bind(&client.external_id)
        .bind(client.active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(ClientError::NotFound(client.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: ClientId,
    ) -> ClientRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM clients WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| ClientError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(ClientError::NotFound(id));
        }
        Ok(())
    }
}

#[async_trait]
impl GroupRepository for SqlxClientRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: GroupId,
    ) -> ClientRepositoryResult<Option<crate::domain::Group>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, office_id, name, display_name, status, staff_id, staff_name, external_id, active, created_date, last_modified_date FROM groups WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_group(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> ClientRepositoryResult<Vec<crate::domain::Group>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, name, display_name, status, staff_id, staff_name, external_id, active, created_date, last_modified_date FROM groups WHERE tenant_id = $1 ORDER BY display_name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_group(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        group: &crate::domain::Group,
    ) -> ClientRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO groups (id, tenant_id, office_id, name, display_name, status, staff_id, staff_name, external_id, active, created_date, last_modified_date) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        )
        .bind(group.id.value())
        .bind(group.tenant_id.value())
        .bind(group.office_id.value())
        .bind(&group.name)
        .bind(&group.display_name)
        .bind(format!("{:?}", group.status))
        .bind(group.staff_id.map(|id| id.value()))
        .bind(&group.staff_name)
        .bind(&group.external_id)
        .bind(group.active)
        .bind(group.created_date)
        .bind(group.last_modified_date)
        .execute(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        group: &crate::domain::Group,
    ) -> ClientRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE groups SET office_id = $2, name = $3, display_name = $4, status = $5, staff_id = $6, staff_name = $7, external_id = $8, active = $9, last_modified_date = $10 WHERE id = $1 AND tenant_id = $11",
        )
        .bind(group.id.value())
        .bind(group.office_id.value())
        .bind(&group.name)
        .bind(&group.display_name)
        .bind(format!("{:?}", group.status))
        .bind(group.staff_id.map(|id| id.value()))
        .bind(&group.staff_name)
        .bind(&group.external_id)
        .bind(group.active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| ClientError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(ClientError::GroupNotFound(group.id));
        }
        Ok(())
    }

    async fn delete(&self, context: &AuthorizedContext, id: GroupId) -> ClientRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM groups WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| ClientError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(ClientError::GroupNotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_client(row: &sqlx::postgres::PgRow) -> ClientRepositoryResult<Client> {
    Ok(Client {
        id: ClientId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            ClientError::QueryError {
                detail: "invalid client id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            ClientError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        office_id: OfficeId::try_from(row.try_get::<i64, _>("office_id")?).map_err(|_| {
            ClientError::QueryError {
                detail: "invalid office id".to_string(),
            }
        })?,
        client_type: match row.try_get::<String, _>("client_type")?.as_str() {
            "Individual" => ClientType::Individual,
            "Group" => ClientType::Group,
            _ => {
                return Err(ClientError::QueryError {
                    detail: "invalid client type".to_string(),
                })
            }
        },
        status: match row.try_get::<String, _>("status")?.as_str() {
            "Active" => ClientStatus::Active,
            "Inactive" => ClientStatus::Inactive,
            "Closed" => ClientStatus::Closed,
            "Pending" => ClientStatus::Pending,
            "Rejected" => ClientStatus::Rejected,
            _ => {
                return Err(ClientError::QueryError {
                    detail: "invalid status".to_string(),
                })
            }
        },
        firstname: row.try_get("firstname")?,
        lastname: row.try_get("lastname")?,
        display_name: row.try_get("display_name")?,
        date_of_birth: row.try_get("date_of_birth")?,
        gender: row.try_get("gender")?,
        mobile_no: row.try_get("mobile_no")?,
        email: row.try_get("email")?,
        office_name: row.try_get("office_name")?,
        staff_id: row
            .try_get::<Option<i64>, _>("staff_id")?
            .map(StaffId::try_from)
            .transpose()
            .map_err(|_| ClientError::QueryError {
                detail: "invalid staff id".to_string(),
            })?,
        staff_name: row.try_get("staff_name")?,
        external_id: row.try_get("external_id")?,
        active: row.try_get("active")?,
        created_date: row.try_get("created_date")?,
        last_modified_date: row.try_get("last_modified_date")?,
    })
}

fn map_row_to_group(row: &sqlx::postgres::PgRow) -> ClientRepositoryResult<crate::domain::Group> {
    Ok(crate::domain::Group {
        id: GroupId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            ClientError::QueryError {
                detail: "invalid group id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            ClientError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        office_id: OfficeId::try_from(row.try_get::<i64, _>("office_id")?).map_err(|_| {
            ClientError::QueryError {
                detail: "invalid office id".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        display_name: row.try_get("display_name")?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "Active" => GroupStatus::Active,
            "Inactive" => GroupStatus::Inactive,
            "Closed" => GroupStatus::Closed,
            _ => {
                return Err(ClientError::QueryError {
                    detail: "invalid status".to_string(),
                })
            }
        },
        staff_id: row
            .try_get::<Option<i64>, _>("staff_id")?
            .map(StaffId::try_from)
            .transpose()
            .map_err(|_| ClientError::QueryError {
                detail: "invalid staff id".to_string(),
            })?,
        staff_name: row.try_get("staff_name")?,
        external_id: row.try_get("external_id")?,
        active: row.try_get("active")?,
        created_date: row.try_get("created_date")?,
        last_modified_date: row.try_get("last_modified_date")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryClientRepository {
        store: Mutex<HashMap<(TenantId, ClientId), Client>>,
    }

    impl Default for InMemoryClientRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryClientRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: ClientId) -> (TenantId, ClientId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl ClientRepository for InMemoryClientRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: ClientId,
        ) -> ClientRepositoryResult<Option<Client>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> ClientRepositoryResult<Vec<Client>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), c)| {
                    if *tid == context.tenant_id {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_all_by_office(
            &self,
            context: &AuthorizedContext,
            office_id: OfficeId,
        ) -> ClientRepositoryResult<Vec<Client>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), c)| {
                    if *tid == context.tenant_id && c.office_id == office_id {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            client: &Client,
        ) -> ClientRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, client.id), client.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            client: &Client,
        ) -> ClientRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, client.id)) {
                store.insert(Self::key(context.tenant_id, client.id), client.clone());
                Ok(())
            } else {
                Err(ClientError::NotFound(client.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: ClientId,
        ) -> ClientRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(ClientError::NotFound(id))
            }
        }
    }

    fn make_context(tenant_id: i64) -> AuthorizedContext {
        AuthorizedContext::new(
            TenantId::new(tenant_id),
            fineract_security::Principal {
                app_user_id: fineract_core::ids::AppUserId::new(1),
                username: "tester".into(),
                roles: HashSet::new(),
                permissions: HashSet::new(),
                office_id: None,
            },
            SessionId::new(),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn client_repository_is_tenant_isolated() {
        let repo = InMemoryClientRepository::new();
        let client = Client::new(
            ClientId::new(42),
            TenantId::new(1),
            fineract_core::ids::OfficeId::new(1),
            crate::domain::ClientType::Individual,
            "John Doe".to_string(),
            Some(fineract_core::ids::StaffId::new(1)),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &client).await.unwrap();
        let result = repo.find_by_id(&ctx, ClientId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().display_name, "John Doe");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, ClientId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
