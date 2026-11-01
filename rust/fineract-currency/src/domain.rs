//! Currency domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::CurrencyId;
use fineract_core::ids::TenantId;

/// A currency supported by the tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Currency {
    pub id: CurrencyId,
    pub tenant_id: TenantId,
    pub code: String,
    pub name: String,
    pub display_symbol: String,
    pub display_label: String,
    pub decimal_places: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Currency {
    pub fn new(
        id: CurrencyId,
        tenant_id: TenantId,
        code: String,
        name: String,
        display_symbol: String,
        display_label: String,
        decimal_places: i32,
    ) -> Result<Self, CurrencyError> {
        if code.trim().is_empty() {
            return Err(CurrencyError::EmptyCode);
        }
        if name.trim().is_empty() {
            return Err(CurrencyError::EmptyName);
        }
        if decimal_places < 0 {
            return Err(CurrencyError::InvalidDecimalPlaces);
        }
        Ok(Self {
            id,
            tenant_id,
            code,
            name,
            display_symbol,
            display_label,
            decimal_places,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CurrencyError {
    #[error("currency code must not be empty")]
    EmptyCode,
    #[error("currency name must not be empty")]
    EmptyName,
    #[error("decimal places must be non-negative")]
    InvalidDecimalPlaces,
    #[error("currency not found: {0}")]
    NotFound(CurrencyId),
    #[error("currency {0} already exists")]
    AlreadyExists(CurrencyId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl CurrencyError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, CurrencyError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for CurrencyError {
    fn from(e: sqlx::Error) -> Self {
        CurrencyError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for CurrencyError {
    fn from(e: fineract_security::SecurityError) -> Self {
        CurrencyError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn currency_creation_validates() {
        let result = Currency::new(
            CurrencyId::new(1),
            TenantId::new(1),
            "".to_string(),
            "US Dollar".to_string(),
            "$".to_string(),
            "USD".to_string(),
            2,
        );
        assert!(matches!(result, Err(CurrencyError::EmptyCode)));

        let result = Currency::new(
            CurrencyId::new(1),
            TenantId::new(1),
            "USD".to_string(),
            "".to_string(),
            "$".to_string(),
            "USD".to_string(),
            2,
        );
        assert!(matches!(result, Err(CurrencyError::EmptyName)));

        let result = Currency::new(
            CurrencyId::new(1),
            TenantId::new(1),
            "USD".to_string(),
            "US Dollar".to_string(),
            "$".to_string(),
            "USD".to_string(),
            -1,
        );
        assert!(matches!(result, Err(CurrencyError::InvalidDecimalPlaces)));

        let currency = Currency::new(
            CurrencyId::new(1),
            TenantId::new(1),
            "USD".to_string(),
            "US Dollar".to_string(),
            "$".to_string(),
            "USD".to_string(),
            2,
        )
        .unwrap();
        assert_eq!(currency.code, "USD");
        assert_eq!(currency.decimal_places, 2);
    }
}
