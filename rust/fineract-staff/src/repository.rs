//! Staff repository.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Staff, StaffError};
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type StaffRepositoryResult<T> = Result<T, StaffError>;

#[async_trait]
pub trait StaffRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: StaffId,
    ) -> StaffRepositoryResult<Option<Staff>>;
    async fn find_by_username(
        &self,
        context: &AuthorizedContext,
        username: &str,
    ) -> StaffRepositoryResult<Option<Staff>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> StaffRepositoryResult<Vec<Staff>>;
    async fn find_all_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> StaffRepositoryResult<Vec<Staff>>;
    async fn insert(&self, context: &AuthorizedContext, staff: &Staff)
        -> StaffRepositoryResult<()>;
    async fn update(&self, context: &AuthorizedContext, staff: &Staff)
        -> StaffRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: StaffId) -> StaffRepositoryResult<()>;
}

pub struct SqlxStaffRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxStaffRepository {
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
    ) -> StaffRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| StaffError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl StaffRepository for SqlxStaffRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: StaffId,
    ) -> StaffRepositoryResult<Option<Staff>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, office_id, firstname, lastname, username, email, mobile_no, is_loan_officer, is_active, password_hash, external_id, created_at, updated_at FROM staff WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| StaffError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_staff(&r)).transpose()
    }

    async fn find_by_username(
        &self,
        context: &AuthorizedContext,
        username: &str,
    ) -> StaffRepositoryResult<Option<Staff>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, office_id, firstname, lastname, username, email, mobile_no, is_loan_officer, is_active, password_hash, external_id, created_at, updated_at FROM staff WHERE username = $1 AND tenant_id = $2",
        )
        .bind(username)
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| StaffError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_staff(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> StaffRepositoryResult<Vec<Staff>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, firstname, lastname, username, email, mobile_no, is_loan_officer, is_active, password_hash, external_id, created_at, updated_at FROM staff WHERE tenant_id = $1 ORDER BY firstname, lastname",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| StaffError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_staff(&r)).collect()
    }

    async fn find_all_by_office(
        &self,
        context: &AuthorizedContext,
        office_id: OfficeId,
    ) -> StaffRepositoryResult<Vec<Staff>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, office_id, firstname, lastname, username, email, mobile_no, is_loan_officer, is_active, password_hash, external_id, created_at, updated_at FROM staff WHERE tenant_id = $1 AND office_id = $2 ORDER BY firstname, lastname",
        )
        .bind(context.tenant_id.value())
        .bind(office_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| StaffError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_staff(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        staff: &Staff,
    ) -> StaffRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO staff (id, tenant_id, office_id, firstname, lastname, username, email, mobile_no, is_loan_officer, is_active, password_hash, external_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(staff.id.value())
        .bind(staff.tenant_id.value())
        .bind(staff.office_id.map(|id| id.value()))
        .bind(&staff.firstname)
        .bind(&staff.lastname)
        .bind(&staff.username)
        .bind(&staff.email)
        .bind(&staff.mobile_no)
        .bind(staff.is_loan_officer)
        .bind(staff.is_active)
        .bind(&staff.password_hash)
        .bind(&staff.external_id)
        .bind(staff.created_at)
        .bind(staff.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| StaffError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        staff: &Staff,
    ) -> StaffRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE staff SET office_id = $2, firstname = $3, lastname = $4, username = $5, email = $6, mobile_no = $7, is_loan_officer = $8, is_active = $9, password_hash = $10, external_id = $11, updated_at = $12 WHERE id = $1 AND tenant_id = $13",
        )
        .bind(staff.id.value())
        .bind(staff.office_id.map(|id| id.value()))
        .bind(&staff.firstname)
        .bind(&staff.lastname)
        .bind(&staff.username)
        .bind(&staff.email)
        .bind(&staff.mobile_no)
        .bind(staff.is_loan_officer)
        .bind(staff.is_active)
        .bind(&staff.password_hash)
        .bind(&staff.external_id)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| StaffError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(StaffError::NotFound(staff.id));
        }
        Ok(())
    }

    async fn delete(&self, context: &AuthorizedContext, id: StaffId) -> StaffRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM staff WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| StaffError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(StaffError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_staff(row: &sqlx::postgres::PgRow) -> StaffRepositoryResult<Staff> {
    Ok(Staff {
        id: StaffId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            StaffError::QueryError {
                detail: "invalid staff id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            StaffError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        office_id: row
            .try_get::<Option<i64>, _>("office_id")?
            .map(OfficeId::try_from)
            .transpose()
            .map_err(|_| StaffError::QueryError {
                detail: "invalid office id".to_string(),
            })?,
        firstname: row.try_get("firstname")?,
        lastname: row.try_get("lastname")?,
        username: row.try_get("username")?,
        email: row.try_get("email")?,
        mobile_no: row.try_get("mobile_no")?,
        is_loan_officer: row.try_get("is_loan_officer")?,
        is_active: row.try_get("is_active")?,
        password_hash: row.try_get("password_hash")?,
        external_id: row.try_get("external_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
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

    pub struct InMemoryStaffRepository {
        store: Mutex<HashMap<(TenantId, StaffId), Staff>>,
    }

    impl Default for InMemoryStaffRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryStaffRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: StaffId) -> (TenantId, StaffId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl StaffRepository for InMemoryStaffRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: StaffId,
        ) -> StaffRepositoryResult<Option<Staff>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_by_username(
            &self,
            context: &AuthorizedContext,
            username: &str,
        ) -> StaffRepositoryResult<Option<Staff>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .values()
                .find(|s| s.tenant_id == context.tenant_id && s.username == username)
                .cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> StaffRepositoryResult<Vec<Staff>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), s)| {
                    if *tid == context.tenant_id {
                        Some(s.clone())
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
        ) -> StaffRepositoryResult<Vec<Staff>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), s)| {
                    if *tid == context.tenant_id && s.office_id == Some(office_id) {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            staff: &Staff,
        ) -> StaffRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, staff.id), staff.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            staff: &Staff,
        ) -> StaffRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, staff.id)) {
                store.insert(Self::key(context.tenant_id, staff.id), staff.clone());
                Ok(())
            } else {
                Err(StaffError::NotFound(staff.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: StaffId,
        ) -> StaffRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(StaffError::NotFound(id))
            }
        }
    }

    fn make_context(tenant_id: i64) -> AuthorizedContext {
        AuthorizedContext::new(
            TenantId::new(tenant_id),
            fineract_security::Principal {
                app_user_id: AppUserId::new(1),
                username: "tester".into(),
                roles: HashSet::new(),
                permissions: HashSet::new(),
                office_id: None,
            },
            SessionId::new(),
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn staff_repository_is_tenant_isolated() {
        let repo = InMemoryStaffRepository::new();
        let staff = Staff::new(
            StaffId::new(42),
            TenantId::new(1),
            "John".to_string(),
            "Doe".to_string(),
            "jdoe".to_string(),
            Some(OfficeId::new(1)),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &staff).await.unwrap();
        let result = repo.find_by_id(&ctx, StaffId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().full_name(), "John Doe");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, StaffId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
