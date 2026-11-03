//! Collateral domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::CollateralId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;

/// Type of collateral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollateralType {
    Property,
    Vehicle,
    Equipment,
    Inventory,
    Receivables,
    CashDeposit,
    Guarantor,
    Other,
}

/// A collateral attached to a loan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collateral {
    pub id: CollateralId,
    pub tenant_id: TenantId,
    pub loan_id: LoanId,
    pub collateral_type: CollateralType,
    pub description: Option<String>,
    pub value: String,
    pub owned_by: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Collateral {
    pub fn new(
        id: CollateralId,
        tenant_id: TenantId,
        loan_id: LoanId,
        collateral_type: CollateralType,
        description: Option<String>,
        value: String,
        owned_by: Option<String>,
    ) -> Result<Self, CollateralError> {
        if value.trim().is_empty() {
            return Err(CollateralError::EmptyValue);
        }
        Ok(Self {
            id,
            tenant_id,
            loan_id,
            collateral_type,
            description,
            value,
            owned_by,
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CollateralError {
    #[error("collateral value must not be empty")]
    EmptyValue,
    #[error("collateral not found: {0}")]
    NotFound(CollateralId),
    #[error("collateral {0} already exists")]
    AlreadyExists(CollateralId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl CollateralError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, CollateralError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for CollateralError {
    fn from(e: sqlx::Error) -> Self {
        CollateralError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for CollateralError {
    fn from(e: fineract_security::SecurityError) -> Self {
        CollateralError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn collateral_creation_validates() {
        let result = Collateral::new(
            CollateralId::new(1),
            TenantId::new(1),
            LoanId::new(1),
            CollateralType::Property,
            None,
            "".to_string(),
            None,
        );
        assert!(matches!(result, Err(CollateralError::EmptyValue)));

        let collateral = Collateral::new(
            CollateralId::new(1),
            TenantId::new(1),
            LoanId::new(1),
            CollateralType::Vehicle,
            Some("2020 Toyota Camry".to_string()),
            "25000".to_string(),
            Some("John Doe".to_string()),
        )
        .unwrap();
        assert_eq!(collateral.collateral_type, CollateralType::Vehicle);
        assert_eq!(collateral.value, "25000");
    }
}
