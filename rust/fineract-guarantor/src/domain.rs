//! Guarantor domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::GuarantorId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;

/// A guarantor for a loan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guarantor {
    pub id: GuarantorId,
    pub tenant_id: TenantId,
    pub loan_id: LoanId,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub identification_number: Option<String>,
    pub net_worth: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Guarantor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: GuarantorId,
        tenant_id: TenantId,
        loan_id: LoanId,
        name: String,
        email: Option<String>,
        phone: Option<String>,
        address: Option<String>,
        identification_number: Option<String>,
        net_worth: Option<String>,
    ) -> Result<Self, GuarantorError> {
        if name.trim().is_empty() {
            return Err(GuarantorError::EmptyName);
        }
        Ok(Self {
            id,
            tenant_id,
            loan_id,
            name,
            email,
            phone,
            address,
            identification_number,
            net_worth,
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GuarantorError {
    #[error("guarantor name must not be empty")]
    EmptyName,
    #[error("guarantor not found: {0}")]
    NotFound(GuarantorId),
    #[error("guarantor {0} already exists")]
    AlreadyExists(GuarantorId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl GuarantorError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, GuarantorError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for GuarantorError {
    fn from(e: sqlx::Error) -> Self {
        GuarantorError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for GuarantorError {
    fn from(e: fineract_security::SecurityError) -> Self {
        GuarantorError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn guarantor_creation_validates() {
        let result = Guarantor::new(
            GuarantorId::new(1),
            TenantId::new(1),
            LoanId::new(1),
            "".to_string(),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(matches!(result, Err(GuarantorError::EmptyName)));

        let guarantor = Guarantor::new(
            GuarantorId::new(1),
            TenantId::new(1),
            LoanId::new(1),
            "John Doe".to_string(),
            Some("john@example.com".to_string()),
            Some("+1234567890".to_string()),
            Some("123 Main St".to_string()),
            Some("ID123456".to_string()),
            Some("1000000".to_string()),
        )
        .unwrap();
        assert_eq!(guarantor.name, "John Doe");
        assert_eq!(guarantor.email, Some("john@example.com".to_string()));
    }
}
