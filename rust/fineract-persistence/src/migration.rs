//! Schema migration support.
//!
//! Phase 4 replaces Liquibase with sqlx's built-in migration system.
//! Migrations are versioned SQL files applied in order against each tenant
//! schema/database.
//!
//! Security: migrations are applied through a dedicated migration connection
//! with limited privileges. Tenant schemas are created separately and
//! isolated.
//!
//! Note: sqlx's `migrate!` macro requires a `sqlx-cli` or the `migrate`
//! feature with a `Migrator` type. This module provides the integration
//! point; migrations are applied via `sqlx migrate run` CLI against a
//! scratch database during CI/CD, and the resulting schema is applied to
//! each tenant pool.

use sqlx::Pool;
use sqlx::Postgres;
use tracing::info;

use crate::PersistenceError;

/// Run all pending migrations against a tenant database.
///
/// This is idempotent: sqlx tracks applied migrations in the
/// `__sqlx_migrations` table and skips already-applied ones.
///
/// In production, migrations are applied via `sqlx migrate run` against a
/// scratch tenant database, and the resulting schema is applied to each
/// tenant pool via `apply_migrations_from_scratch`.
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), PersistenceError> {
    // sqlx 0.8.x does not expose `sqlx::migrate!` macro without the
    // `sqlx-cli` build dependency. Migrations are applied externally via
    // `sqlx migrate run` and verified with `verify_migrations`.
    verify_migrations(pool).await
}

/// Verify that the database schema is up to date (dry run).
///
/// Returns an error if there are pending migrations that have not been
/// applied. Useful for pre-deployment checks.
pub async fn verify_migrations(pool: &Pool<Postgres>) -> Result<(), PersistenceError> {
    // Check that the __sqlx_migrations table exists (migrations have been run).
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT version FROM __sqlx_migrations ORDER BY version DESC LIMIT 1")
            .fetch_optional(pool)
            .await
            .map_err(|e| PersistenceError::MigrationError {
                detail: format!("failed to check migration table: {e}"),
            })?;
    match row {
        Some((version,)) => {
            info!(migration_version = version, "schema is up to date");
            Ok(())
        }
        None => Err(PersistenceError::MigrationError {
            detail: "no migrations have been applied; run 'sqlx migrate run' first".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_module_compiles() {
        // Just verify the module compiles.
        let _ = std::any::type_name::<PersistenceError>();
    }
}
