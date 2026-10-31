//! Document domain models.
//! Document metadata and storage references.

use serde::{Deserialize, Serialize};

use fineract_core::ids::DocumentId;
use fineract_core::ids::TenantId;

/// A document reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub tenant_id: TenantId,
    pub name: String,
    pub content_type: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub active: bool,
}

impl Document {
    pub fn new(
        id: DocumentId,
        tenant_id: TenantId,
        name: String,
        content_type: String,
        storage_path: String,
        size_bytes: i64,
    ) -> Result<Self, DocumentError> {
        if name.trim().is_empty() {
            return Err(DocumentError::EmptyName);
        }
        if content_type.trim().is_empty() {
            return Err(DocumentError::EmptyContentType);
        }
        if size_bytes < 0 {
            return Err(DocumentError::InvalidSize);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            content_type,
            storage_path,
            size_bytes,
            active: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocumentError {
    #[error("document name must not be empty")]
    EmptyName,
    #[error("document not found: {0}")]
    NotFound(DocumentId),
    #[error("document {0} already exists")]
    AlreadyExists(DocumentId),
    #[error("content type must not be empty")]
    EmptyContentType,
    #[error("document size must not be negative")]
    InvalidSize,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl DocumentError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, DocumentError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for DocumentError {
    fn from(e: sqlx::Error) -> Self {
        DocumentError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for DocumentError {
    fn from(e: fineract_security::SecurityError) -> Self {
        DocumentError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_creation_validates() {
        let result = Document::new(
            DocumentId::new(1),
            TenantId::new(1),
            "".to_string(),
            "application/pdf".to_string(),
            "/docs/1.pdf".to_string(),
            1024,
        );
        assert!(matches!(result, Err(DocumentError::EmptyName)));

        let doc = Document::new(
            DocumentId::new(1),
            TenantId::new(1),
            "Loan Agreement".to_string(),
            "application/pdf".to_string(),
            "/docs/1.pdf".to_string(),
            1024,
        )
        .unwrap();
        assert_eq!(doc.name, "Loan Agreement");
        assert_eq!(doc.size_bytes, 1024);
    }
}
