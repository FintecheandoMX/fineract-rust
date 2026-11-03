//! Note domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::NoteId;
use fineract_core::ids::TenantId;

/// Type of entity the note is attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteEntityType {
    Client,
    Loan,
    Savings,
    Group,
    Center,
}

/// A note attached to an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub tenant_id: TenantId,
    pub entity_type: NoteEntityType,
    pub entity_id: i64,
    pub note: String,
    pub created_by: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub active: bool,
}

impl Note {
    pub fn new(
        id: NoteId,
        tenant_id: TenantId,
        entity_type: NoteEntityType,
        entity_id: i64,
        note: String,
        created_by: i64,
    ) -> Result<Self, NoteError> {
        if note.trim().is_empty() {
            return Err(NoteError::EmptyNote);
        }
        if entity_id <= 0 {
            return Err(NoteError::InvalidEntityId);
        }
        Ok(Self {
            id,
            tenant_id,
            entity_type,
            entity_id,
            note,
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: None,
            active: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoteError {
    #[error("note must not be empty")]
    EmptyNote,
    #[error("entity id must be positive")]
    InvalidEntityId,
    #[error("note not found: {0}")]
    NotFound(NoteId),
    #[error("note {0} already exists")]
    AlreadyExists(NoteId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl NoteError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, NoteError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for NoteError {
    fn from(e: sqlx::Error) -> Self {
        NoteError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for NoteError {
    fn from(e: fineract_security::SecurityError) -> Self {
        NoteError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn note_creation_validates() {
        let result = Note::new(
            NoteId::new(1),
            TenantId::new(1),
            NoteEntityType::Client,
            1,
            "".to_string(),
            1,
        );
        assert!(matches!(result, Err(NoteError::EmptyNote)));

        let result = Note::new(
            NoteId::new(1),
            TenantId::new(1),
            NoteEntityType::Client,
            0,
            "Test note".to_string(),
            1,
        );
        assert!(matches!(result, Err(NoteError::InvalidEntityId)));

        let note = Note::new(
            NoteId::new(1),
            TenantId::new(1),
            NoteEntityType::Loan,
            100,
            "Loan note".to_string(),
            1,
        )
        .unwrap();
        assert_eq!(note.entity_type, NoteEntityType::Loan);
        assert_eq!(note.entity_id, 100);
    }
}
