//! Code and CodeValue domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::CodeId;
use fineract_core::ids::CodeValueId;
use fineract_core::ids::TenantId;

/// A code (lookup category) containing multiple code values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Code {
    pub id: CodeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Code {
    pub fn new(
        id: CodeId,
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
    ) -> Result<Self, CodeError> {
        if name.trim().is_empty() {
            return Err(CodeError::EmptyName);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            description,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

/// A code value (lookup item) belonging to a code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeValue {
    pub id: CodeValueId,
    pub code_id: CodeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub position: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CodeValue {
    pub fn new(
        id: CodeValueId,
        code_id: CodeId,
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
        position: i32,
    ) -> Result<Self, CodeError> {
        if name.trim().is_empty() {
            return Err(CodeError::EmptyName);
        }
        Ok(Self {
            id,
            code_id,
            tenant_id,
            name,
            description,
            position,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CodeError {
    #[error("code name must not be empty")]
    EmptyName,
    #[error("code not found: {0}")]
    NotFound(CodeId),
    #[error("code value not found: {0}")]
    CodeValueNotFound(CodeValueId),
    #[error("code {0} already exists")]
    AlreadyExists(CodeId),
    #[error("code value {0} already exists")]
    CodeValueAlreadyExists(CodeValueId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl CodeError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, CodeError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for CodeError {
    fn from(e: sqlx::Error) -> Self {
        CodeError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for CodeError {
    fn from(e: fineract_security::SecurityError) -> Self {
        CodeError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn code_creation_validates() {
        let result = Code::new(CodeId::new(1), TenantId::new(1), "".to_string(), None);
        assert!(matches!(result, Err(CodeError::EmptyName)));

        let code = Code::new(
            CodeId::new(1),
            TenantId::new(1),
            "Gender".to_string(),
            Some("Gender codes".to_string()),
        )
        .unwrap();
        assert_eq!(code.name, "Gender");
    }

    #[test]
    fn code_value_creation_validates() {
        let result = CodeValue::new(
            CodeValueId::new(1),
            CodeId::new(1),
            TenantId::new(1),
            "".to_string(),
            None,
            1,
        );
        assert!(matches!(result, Err(CodeError::EmptyName)));

        let cv = CodeValue::new(
            CodeValueId::new(1),
            CodeId::new(1),
            TenantId::new(1),
            "Male".to_string(),
            None,
            1,
        )
        .unwrap();
        assert_eq!(cv.name, "Male");
    }
}
