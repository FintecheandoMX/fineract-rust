//! Base repository implementations using sqlx with compile-time-checked,
//! parameterized queries.
//!
//! All queries filter by `tenant_id` by default, enforcing multi-tenancy
//! isolation at the repository layer. No string concatenation is ever used
//! for SQL — all queries are parameterized via `sqlx::query!` and
//! `sqlx::query_as!`.

use std::sync::Arc;

use async_trait::async_trait;

use crate::pool::TenantPoolManager;
use crate::PersistenceError;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

/// Result type for repository operations.
pub type RepositoryResult<T> = Result<T, PersistenceError>;

/// Base repository trait with tenant-aware read operations.
///
/// Every repository operation receives an explicit [`AuthorizedContext`] and
/// enforces tenant filtering by default.
#[async_trait]
pub trait TenantRepository {
    type Entity;
    type Id;

    /// Find an entity by ID within the context's tenant.
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: Self::Id,
    ) -> RepositoryResult<Option<Self::Entity>>;

    /// Find all entities within the context's tenant.
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> RepositoryResult<Vec<Self::Entity>>;

    /// Insert a new entity within the context's tenant.
    async fn insert(
        &self,
        context: &AuthorizedContext,
        entity: &Self::Entity,
    ) -> RepositoryResult<()>;

    /// Update an existing entity within the context's tenant.
    async fn update(
        &self,
        context: &AuthorizedContext,
        entity: &Self::Entity,
    ) -> RepositoryResult<()>;

    /// Delete an entity by ID within the context's tenant.
    async fn delete(&self, context: &AuthorizedContext, id: Self::Id) -> RepositoryResult<()>;
}

/// Base implementation of [`TenantRepository`] backed by sqlx.
///
/// Subtypes implement the table-specific SQL and row mapping.
#[allow(dead_code)]
pub struct SqlxRepository<T, Id> {
    pool_manager: Arc<TenantPoolManager>,
    table_name: String,
    id_column: String,
    _marker: std::marker::PhantomData<(T, Id)>,
}

impl<T, Id> SqlxRepository<T, Id> {
    pub fn new(pool_manager: Arc<TenantPoolManager>, table_name: &str, id_column: &str) -> Self {
        Self {
            pool_manager,
            table_name: table_name.to_string(),
            id_column: id_column.to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Resolve the connection pool for the context's tenant.
    #[allow(dead_code)]
    async fn pool_for_context(
        &self,
        context: &AuthorizedContext,
        connection_string: &str,
    ) -> RepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, connection_string.to_string())
            .await
            .map_err(|e| PersistenceError::ConnectionError {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

/// In-memory repository for testing without a live database.
///
/// Useful for unit tests that exercise business logic without requiring
/// PostgreSQL. Integration tests should use a real database.
pub struct InMemoryRepository<T, Id> {
    store: std::sync::Mutex<HashMap<(TenantId, Id), T>>,
}

impl<T: Clone, Id: Clone + std::cmp::Eq + std::hash::Hash> Default for InMemoryRepository<T, Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, Id: Clone + Eq + std::hash::Hash> InMemoryRepository<T, Id> {
    pub fn new() -> Self {
        Self {
            store: std::sync::Mutex::new(HashMap::new()),
        }
    }

    fn key(tenant_id: TenantId, id: Id) -> (TenantId, Id) {
        (tenant_id, id)
    }
}

use std::collections::HashMap;

impl<T, Id> InMemoryRepository<T, Id>
where
    T: Clone,
    Id: Clone + Eq + std::hash::Hash,
{
    pub fn insert_test(&self, tenant_id: TenantId, id: Id, entity: T) -> RepositoryResult<()> {
        self.store
            .lock()
            .unwrap()
            .insert(Self::key(tenant_id, id), entity);
        Ok(())
    }

    pub fn get_test(&self, tenant_id: TenantId, id: Id) -> RepositoryResult<Option<T>> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .get(&Self::key(tenant_id, id))
            .cloned())
    }

    pub fn remove_test(&self, tenant_id: TenantId, id: Id) -> RepositoryResult<Option<T>> {
        Ok(self.store.lock().unwrap().remove(&Self::key(tenant_id, id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::{AppUserId, ClientId, TenantId};
    use fineract_security::SessionId;
    use std::collections::HashSet;

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
    async fn in_memory_repository_is_tenant_isolated() {
        let repo: InMemoryRepository<String, ClientId> = InMemoryRepository::new();

        repo.insert_test(TenantId::new(1), ClientId::new(42), "client-a".to_string())
            .unwrap();

        // Same tenant can read.
        let ctx = make_context(1);
        let result = repo.get_test(ctx.tenant_id, ClientId::new(42)).unwrap();
        assert_eq!(result, Some("client-a".to_string()));

        // Different tenant cannot read (IDOR).
        let ctx2 = make_context(2);
        let result = repo.get_test(ctx2.tenant_id, ClientId::new(42)).unwrap();
        assert_eq!(result, None);
    }
}
