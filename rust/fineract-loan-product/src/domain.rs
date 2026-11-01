//! Loan product domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::LoanProductId;
use fineract_core::ids::TenantId;

/// A loan product definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanProduct {
    pub id: LoanProductId,
    pub tenant_id: TenantId,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub principal: String,
    pub number_of_repayments: i32,
    pub repayment_every: i32,
    pub repayment_frequency_type: i32,
    pub amortization_type: i32,
    pub interest_rate_per_period: String,
    pub interest_type: i32,
    pub interest_calculation_period_type: i32,
    pub is_equal_amortization: bool,
    pub min_principal: String,
    pub max_principal: String,
    pub min_number_of_repayments: i32,
    pub max_number_of_repayments: i32,
    pub min_interest_rate_per_period: String,
    pub max_interest_rate_per_period: String,
    pub currency_code: String,
    pub currency_digits: i32,
    pub in_arrears_tolerance: i32,
    pub days_in_year_type: i32,
    pub days_in_month_type: i32,
    pub include_in_borrower_cycle: bool,
    pub use_borrower_cycle: bool,
    pub number_of_borrower_cycle: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl LoanProduct {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: LoanProductId,
        tenant_id: TenantId,
        name: String,
        short_name: String,
        description: Option<String>,
        principal: String,
        number_of_repayments: i32,
        repayment_every: i32,
        repayment_frequency_type: i32,
        amortization_type: i32,
        interest_rate_per_period: String,
        interest_type: i32,
        interest_calculation_period_type: i32,
        currency_code: String,
    ) -> Result<Self, LoanProductError> {
        if name.trim().is_empty() {
            return Err(LoanProductError::EmptyName);
        }
        if short_name.trim().is_empty() {
            return Err(LoanProductError::EmptyShortName);
        }
        if currency_code.trim().is_empty() {
            return Err(LoanProductError::EmptyCurrencyCode);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            short_name,
            description,
            principal,
            number_of_repayments,
            repayment_every,
            repayment_frequency_type,
            amortization_type,
            interest_rate_per_period,
            interest_type,
            interest_calculation_period_type,
            is_equal_amortization: true,
            min_principal: "0".to_string(),
            max_principal: "100000000".to_string(),
            min_number_of_repayments: 1,
            max_number_of_repayments: 1000,
            min_interest_rate_per_period: "0".to_string(),
            max_interest_rate_per_period: "100".to_string(),
            currency_code,
            currency_digits: 2,
            in_arrears_tolerance: 0,
            days_in_year_type: 1,
            days_in_month_type: 1,
            include_in_borrower_cycle: false,
            use_borrower_cycle: false,
            number_of_borrower_cycle: 0,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LoanProductError {
    #[error("loan product name must not be empty")]
    EmptyName,
    #[error("loan product short name must not be empty")]
    EmptyShortName,
    #[error("currency code must not be empty")]
    EmptyCurrencyCode,
    #[error("loan product not found: {0}")]
    NotFound(LoanProductId),
    #[error("loan product {0} already exists")]
    AlreadyExists(LoanProductId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl LoanProductError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, LoanProductError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for LoanProductError {
    fn from(e: sqlx::Error) -> Self {
        LoanProductError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for LoanProductError {
    fn from(e: fineract_security::SecurityError) -> Self {
        LoanProductError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn loan_product_creation_validates() {
        let result = LoanProduct::new(
            LoanProductId::new(1),
            TenantId::new(1),
            "".to_string(),
            "Test".to_string(),
            None,
            "1000".to_string(),
            12,
            1,
            2,
            1,
            "10".to_string(),
            1,
            1,
            "USD".to_string(),
        );
        assert!(matches!(result, Err(LoanProductError::EmptyName)));

        let result = LoanProduct::new(
            LoanProductId::new(1),
            TenantId::new(1),
            "Test".to_string(),
            "".to_string(),
            None,
            "1000".to_string(),
            12,
            1,
            2,
            1,
            "10".to_string(),
            1,
            1,
            "USD".to_string(),
        );
        assert!(matches!(result, Err(LoanProductError::EmptyShortName)));

        let result = LoanProduct::new(
            LoanProductId::new(1),
            TenantId::new(1),
            "Test".to_string(),
            "Test".to_string(),
            None,
            "1000".to_string(),
            12,
            1,
            2,
            1,
            "10".to_string(),
            1,
            1,
            "".to_string(),
        );
        assert!(matches!(result, Err(LoanProductError::EmptyCurrencyCode)));

        let product = LoanProduct::new(
            LoanProductId::new(1),
            TenantId::new(1),
            "Test Loan".to_string(),
            "TL".to_string(),
            None,
            "1000".to_string(),
            12,
            1,
            2,
            1,
            "10".to_string(),
            1,
            1,
            "USD".to_string(),
        )
        .unwrap();
        assert_eq!(product.name, "Test Loan");
    }
}
