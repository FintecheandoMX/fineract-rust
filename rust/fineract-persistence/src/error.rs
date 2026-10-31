//! Persistence errors.

use thiserror::Error;

/// Error produced by the persistence layer.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PersistenceError {
    #[error("connection error: {detail}")]
    ConnectionError { detail: String },

    #[error("query error: {detail}")]
    QueryError { detail: String },

    #[error("migration error: {detail}")]
    MigrationError { detail: String },

    #[error("not found: {entity} {id}")]
    NotFound { entity: String, id: String },

    #[error("tenant isolation violation")]
    TenantIsolationViolation,

    #[error("internal persistence error: {detail}")]
    Internal { detail: String },

    #[error("sqlx error: {detail}")]
    Sqlx { detail: String },
}

impl From<sqlx::Error> for PersistenceError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => PersistenceError::QueryError {
                detail: "row not found".to_string(),
            },
            _ => PersistenceError::Sqlx {
                detail: e.to_string(),
            },
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::ConnectionError {
            detail: e.to_string(),
        }
    }
}
