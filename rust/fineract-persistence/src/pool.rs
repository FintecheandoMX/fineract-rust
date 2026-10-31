//! Tenant-aware connection pool management.
//!
//! Each tenant gets its own dedicated [`sqlx::Pool<Postgres>`], ensuring
//! physical isolation of tenant data at the connection level. Pools are
//! created lazily on first access and cached for reuse.
//!
//! Security: tenant pools are never shared across tenants. The pool manager
//! is the single point of tenant-to-pool mapping, preventing tenant leakage
//! through connection reuse.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::{debug, info};

use crate::PersistenceError;
use fineract_core::ids::TenantId;

/// Configuration for tenant connection pools.
#[derive(Debug, Clone)]
pub struct TenantPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for TenantPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

/// Manages one connection pool per tenant.
///
/// All tenant data access goes through this manager. Pools are keyed by
/// `TenantId` so that tenant A can never accidentally use Tenant B's
/// connection pool.
#[derive(Debug, Clone)]
pub struct TenantPoolManager {
    pools: Arc<tokio::sync::RwLock<HashMap<TenantId, Pool<Postgres>>>>,
    config: TenantPoolConfig,
}

impl TenantPoolManager {
    /// Create a new manager with default config.
    pub fn new() -> Self {
        Self::with_config(TenantPoolConfig::default())
    }

    /// Create a new manager with custom config.
    pub fn with_config(config: TenantPoolConfig) -> Self {
        Self {
            pools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get or create the connection pool for a tenant.
    ///
    /// Lazily creates the pool on first access. Returns an error if the
    /// connection string is invalid or the database is unreachable.
    pub async fn pool_for_tenant(
        &self,
        tenant_id: TenantId,
        connection_string: String,
    ) -> Result<Pool<Postgres>, PersistenceError> {
        // Fast path: check if pool already exists.
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(&tenant_id) {
                debug!(tenant = %tenant_id, "reusing existing tenant pool");
                return Ok(pool.clone());
            }
        }

        // Slow path: create a new pool.
        let tenant_id_clone = tenant_id;
        let config = self.config.clone();
        let connection_string_owned = connection_string;

        let pool = tokio::task::spawn_blocking(move || {
            PgPoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(config.acquire_timeout)
                .idle_timeout(config.idle_timeout)
                .max_lifetime(config.max_lifetime)
                .connect_lazy(&connection_string_owned)
        })
        .await
        .map_err(|e| PersistenceError::Internal {
            detail: format!("task join error creating pool: {e}"),
        })?
        .map_err(|e| PersistenceError::ConnectionError {
            detail: format!(
                "failed to create pool for tenant {}: {e}",
                tenant_id_clone.value()
            ),
        })?;

        info!(tenant = %tenant_id_clone, "created new tenant connection pool");

        // Store the pool (check again in case another task created it first).
        let mut pools = self.pools.write().await;
        if let Some(existing) = pools.get(&tenant_id) {
            Ok(existing.clone())
        } else {
            pools.insert(tenant_id_clone, pool.clone());
            Ok(pool)
        }
    }

    /// Close and remove a tenant's pool.
    pub async fn close_pool_for_tenant(&self, tenant_id: TenantId) {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.remove(&tenant_id) {
            pool.close().await;
            info!(tenant = %tenant_id, "closed tenant connection pool");
        }
    }

    /// Returns the number of active tenant pools.
    pub async fn pool_count(&self) -> usize {
        self.pools.read().await.len()
    }

    /// Close all tenant pools.
    pub async fn close_all(&self) {
        let mut pools = self.pools.write().await;
        for (tenant_id, pool) in pools.drain() {
            pool.close().await;
            debug!(tenant = %tenant_id, "closed tenant pool during shutdown");
        }
        info!("closed all tenant connection pools");
    }
}

impl Default for TenantPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_manager_default_config() {
        let manager = TenantPoolManager::new();
        let config = TenantPoolConfig::default();
        assert_eq!(manager.config.max_connections, config.max_connections);
        assert_eq!(manager.config.min_connections, config.min_connections);
    }

    #[test]
    fn pool_manager_with_config() {
        let config = TenantPoolConfig {
            max_connections: 5,
            ..Default::default()
        };
        let manager = TenantPoolManager::with_config(config.clone());
        assert_eq!(manager.config.max_connections, 5);
    }

    #[tokio::test]
    async fn pool_count_starts_at_zero() {
        let manager = TenantPoolManager::new();
        assert_eq!(manager.pool_count().await, 0);
    }
}
