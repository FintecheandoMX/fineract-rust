//! Tax domain models.
//! Tax components with rates and inclusion/exclusion rules.

use serde::{Deserialize, Serialize};

use fineract_core::ids::TaxId;
use fineract_core::ids::TenantId;

/// Whether tax is included in the amount or added on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxInclusion {
    /// Tax is included in the amount (price includes tax).
    Included,
    /// Tax is added on top of the amount.
    Excluded,
}

/// A tax component definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tax {
    pub id: TaxId,
    pub tenant_id: TenantId,
    pub name: String,
    pub rate: rust_decimal::Decimal,
    pub inclusion: TaxInclusion,
    pub active: bool,
}

impl Tax {
    pub fn new(
        id: TaxId,
        tenant_id: TenantId,
        name: String,
        rate: rust_decimal::Decimal,
        inclusion: TaxInclusion,
    ) -> Result<Self, TaxError> {
        if name.trim().is_empty() {
            return Err(TaxError::EmptyName);
        }
        if rate.is_zero() || rate.is_sign_negative() {
            return Err(TaxError::InvalidValue);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            rate,
            inclusion,
            active: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TaxError {
    #[error("tax name must not be empty")]
    EmptyName,
    #[error("tax not found: {0}")]
    NotFound(TaxId),
    #[error("tax {0} already exists")]
    AlreadyExists(TaxId),
    #[error("tax rate must be positive and non-zero")]
    InvalidValue,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl TaxError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, TaxError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for TaxError {
    fn from(e: sqlx::Error) -> Self {
        TaxError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for TaxError {
    fn from(e: fineract_security::SecurityError) -> Self {
        TaxError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tax_creation_validates() {
        let result = Tax::new(
            TaxId::new(1),
            TenantId::new(1),
            "".to_string(),
            rust_decimal::Decimal::new(1000, 2), // 10.00%
            TaxInclusion::Excluded,
        );
        assert!(matches!(result, Err(TaxError::EmptyName)));

        let tax = Tax::new(
            TaxId::new(1),
            TenantId::new(1),
            "VAT".to_string(),
            rust_decimal::Decimal::new(1000, 2),
            TaxInclusion::Excluded,
        )
        .unwrap();
        assert_eq!(tax.name, "VAT");
        assert_eq!(tax.inclusion, TaxInclusion::Excluded);
    }
}
