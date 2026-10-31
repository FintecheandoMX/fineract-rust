//! Rates domain models.
//! Interest rates and rate tiers for loan/savings products.

use serde::{Deserialize, Serialize};

use fineract_core::ids::RateId;
use fineract_core::ids::TenantId;

/// Period type for rate application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatePeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// A rate definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rate {
    pub id: RateId,
    pub tenant_id: TenantId,
    pub name: String,
    pub period: RatePeriod,
    pub annual_percentage_rate: rust_decimal::Decimal,
    pub active: bool,
}

impl Rate {
    pub fn new(
        id: RateId,
        tenant_id: TenantId,
        name: String,
        period: RatePeriod,
        apr: rust_decimal::Decimal,
    ) -> Result<Self, RateError> {
        if name.trim().is_empty() {
            return Err(RateError::EmptyName);
        }
        if apr.is_zero() || apr.is_sign_negative() {
            return Err(RateError::InvalidValue);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            period,
            annual_percentage_rate: apr,
            active: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RateError {
    #[error("rate name must not be empty")]
    EmptyName,
    #[error("rate not found: {0}")]
    NotFound(RateId),
    #[error("rate {0} already exists")]
    AlreadyExists(RateId),
    #[error("rate value must be positive and non-zero")]
    InvalidValue,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl RateError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, RateError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for RateError {
    fn from(e: sqlx::Error) -> Self {
        RateError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for RateError {
    fn from(e: fineract_security::SecurityError) -> Self {
        RateError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_creation_validates() {
        let result = Rate::new(
            RateId::new(1),
            TenantId::new(1),
            "".to_string(),
            RatePeriod::Yearly,
            rust_decimal::Decimal::new(500, 2), // 5.00%
        );
        assert!(matches!(result, Err(RateError::EmptyName)));

        let rate = Rate::new(
            RateId::new(1),
            TenantId::new(1),
            "Standard Annual".to_string(),
            RatePeriod::Yearly,
            rust_decimal::Decimal::new(500, 2),
        )
        .unwrap();
        assert_eq!(rate.name, "Standard Annual");
        assert_eq!(rate.period, RatePeriod::Yearly);
    }
}
