//! Charge domain models.
//! Charges are fees applied to loan or savings products (percentage or fixed).

use serde::{Deserialize, Serialize};

use fineract_core::ids::ChargeId;
use fineract_core::ids::TenantId;

/// Type of charge calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargeType {
    /// Fixed monetary amount.
    Fixed,
    /// Percentage of the principal/transaction amount.
    Percentage,
}

/// A charge definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Charge {
    pub id: ChargeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub charge_type: ChargeType,
    pub value: rust_decimal::Decimal,
    pub active: bool,
}

impl Charge {
    pub fn new(
        id: ChargeId,
        tenant_id: TenantId,
        name: String,
        charge_type: ChargeType,
        value: rust_decimal::Decimal,
    ) -> Result<Self, ChargeError> {
        if name.trim().is_empty() {
            return Err(ChargeError::EmptyName);
        }
        if value.is_zero() || value.is_sign_negative() {
            return Err(ChargeError::InvalidValue);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            charge_type,
            value,
            active: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChargeError {
    #[error("charge name must not be empty")]
    EmptyName,
    #[error("charge not found: {0}")]
    NotFound(ChargeId),
    #[error("charge {0} already exists")]
    AlreadyExists(ChargeId),
    #[error("charge value must be positive and non-zero")]
    InvalidValue,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl ChargeError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, ChargeError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for ChargeError {
    fn from(e: sqlx::Error) -> Self {
        ChargeError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for ChargeError {
    fn from(e: fineract_security::SecurityError) -> Self {
        ChargeError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_creation_validates() {
        let result = Charge::new(
            ChargeId::new(1),
            TenantId::new(1),
            "".to_string(),
            ChargeType::Fixed,
            rust_decimal::Decimal::new(100, 2),
        );
        assert!(matches!(result, Err(ChargeError::EmptyName)));

        let charge = Charge::new(
            ChargeId::new(1),
            TenantId::new(1),
            "Processing Fee".to_string(),
            ChargeType::Fixed,
            rust_decimal::Decimal::new(100, 2),
        )
        .unwrap();
        assert_eq!(charge.name, "Processing Fee");
        assert_eq!(charge.charge_type, ChargeType::Fixed);
    }
}
