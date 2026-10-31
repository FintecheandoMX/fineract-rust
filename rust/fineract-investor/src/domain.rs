//! Investor domain models.
//! Investor accounts and investment products.

use serde::{Deserialize, Serialize};

use fineract_core::ids::InvestorId;
use fineract_core::ids::TenantId;

/// An investor account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Investor {
    pub id: InvestorId,
    pub tenant_id: TenantId,
    pub name: String,
    pub external_id: Option<String>,
    pub active: bool,
}

impl Investor {
    pub fn new(
        id: InvestorId,
        tenant_id: TenantId,
        name: String,
        external_id: Option<String>,
    ) -> Result<Self, InvestorError> {
        if name.trim().is_empty() {
            return Err(InvestorError::EmptyName);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            external_id,
            active: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvestorError {
    #[error("investor name must not be empty")]
    EmptyName,
    #[error("investor not found: {0}")]
    NotFound(InvestorId),
    #[error("investor {0} already exists")]
    AlreadyExists(InvestorId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl InvestorError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, InvestorError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for InvestorError {
    fn from(e: sqlx::Error) -> Self {
        InvestorError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for InvestorError {
    fn from(e: fineract_security::SecurityError) -> Self {
        InvestorError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn investor_creation_validates() {
        let result = Investor::new(InvestorId::new(1), TenantId::new(1), "".to_string(), None);
        assert!(matches!(result, Err(InvestorError::EmptyName)));

        let investor = Investor::new(
            InvestorId::new(1),
            TenantId::new(1),
            "Capital Partners".to_string(),
            Some("EXT-001".to_string()),
        )
        .unwrap();
        assert_eq!(investor.name, "Capital Partners");
        assert_eq!(investor.external_id, Some("EXT-001".to_string()));
    }
}
