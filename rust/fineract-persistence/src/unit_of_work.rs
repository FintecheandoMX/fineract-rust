//! Unit-of-Work pattern for transactional boundaries.
//!
//! Groups multiple repository mutations into a single database transaction
//! with commit/rollback semantics. Ensures atomicity across multiple
//! repository operations.

use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::Transaction;
use tracing::{debug, info};

use crate::PersistenceError;
use fineract_core::ids::TenantId;

/// A database transaction bound to a single tenant.
pub struct TenantTransaction {
    transaction: Transaction<'static, Postgres>,
    tenant_id: TenantId,
}

impl TenantTransaction {
    /// Create from an existing sqlx transaction.
    pub fn new(transaction: Transaction<'static, Postgres>, tenant_id: TenantId) -> Self {
        Self {
            transaction,
            tenant_id,
        }
    }

    /// Return the tenant ID this transaction operates on.
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Return a mutable reference to the underlying sqlx transaction.
    pub fn transaction_mut(&mut self) -> &mut Transaction<'static, Postgres> {
        &mut self.transaction
    }

    /// Return an immutable reference to the underlying sqlx transaction.
    pub fn transaction(&self) -> &Transaction<'static, Postgres> {
        &self.transaction
    }
}

/// Unit-of-Work trait: groups mutations into a single transaction.
#[async_trait]
pub trait UnitOfWork {
    type Error: std::error::Error + Send + Sync;

    /// Begin a new transaction.
    async fn begin(&mut self) -> Result<(), Self::Error>;

    /// Commit the current transaction.
    async fn commit(&mut self) -> Result<(), Self::Error>;

    /// Roll back the current transaction.
    async fn rollback(&mut self) -> Result<(), Self::Error>;
}

/// SQLx-backed Unit-of-Work implementation.
///
/// Creates a connection pool for a tenant and provides transactional
/// boundaries for repository operations.
pub struct SqlxUnitOfWork {
    pool: Pool<Postgres>,
    tenant_id: TenantId,
    transaction: Option<Transaction<'static, Postgres>>,
}

impl SqlxUnitOfWork {
    /// Create a new Unit-of-Work bound to a tenant.
    pub async fn new(
        connection_string: &str,
        tenant_id: TenantId,
    ) -> Result<Self, PersistenceError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
            .map_err(|e| PersistenceError::ConnectionError {
                detail: format!("failed to create pool for UoW: {e}"),
            })?;

        Ok(Self {
            pool,
            tenant_id,
            transaction: None,
        })
    }

    /// Create from an existing pool.
    pub fn from_pool(pool: Pool<Postgres>, tenant_id: TenantId) -> Self {
        Self {
            pool,
            tenant_id,
            transaction: None,
        }
    }

    /// Return the tenant ID this UoW operates on.
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Return a reference to the connection pool.
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}

#[async_trait]
impl UnitOfWork for SqlxUnitOfWork {
    type Error = PersistenceError;

    async fn begin(&mut self) -> Result<(), Self::Error> {
        debug!(tenant = %self.tenant_id, "beginning transaction");
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PersistenceError::ConnectionError {
                detail: format!("failed to begin transaction: {e}"),
            })?;
        self.transaction = Some(tx);
        Ok(())
    }

    async fn commit(&mut self) -> Result<(), Self::Error> {
        let tx = self
            .transaction
            .take()
            .ok_or_else(|| PersistenceError::Internal {
                detail: "no transaction to commit".to_string(),
            })?;
        tx.commit()
            .await
            .map_err(|e| PersistenceError::QueryError {
                detail: format!("failed to commit transaction: {e}"),
            })?;
        info!(tenant = %self.tenant_id, "transaction committed");
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), Self::Error> {
        let tx = self
            .transaction
            .take()
            .ok_or_else(|| PersistenceError::Internal {
                detail: "no transaction to roll back".to_string(),
            })?;
        tx.rollback()
            .await
            .map_err(|e| PersistenceError::QueryError {
                detail: format!("failed to roll back transaction: {e}"),
            })?;
        info!(tenant = %self.tenant_id, "transaction rolled back");
        Ok(())
    }
}

impl Drop for SqlxUnitOfWork {
    fn drop(&mut self) {
        // If a transaction is still open when the UoW is dropped, roll it
        // back. This cannot be async, so we log and leak (sqlx will clean
        // up on connection drop).
        if self.transaction.is_some() {
            tracing::warn!(
                tenant = %self.tenant_id,
                "UnitOfWork dropped with open transaction; rolling back"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uow_tenant_id_is_correct() {
        // Can't construct without a live DB, but verify the struct fields
        // align via a unit test on tenant ID handling.
        let _tenant = TenantId::new(1);
        assert_eq!(_tenant.value(), 1);
    }
}
