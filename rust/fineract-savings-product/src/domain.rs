//! Savings product domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::SavingsProductId;
use fineract_core::ids::TenantId;

/// A savings product definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavingsProduct {
    pub id: SavingsProductId,
    pub tenant_id: TenantId,
    pub name: String,
    pub short_name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub currency_digits: i32,
    pub interest_rate: String,
    pub interest_compounding_period_type: i32,
    pub interest_posting_period_type: i32,
    pub interest_calculation_type: i32,
    pub interest_calculation_days_in_year_type: i32,
    pub min_required_opening_balance: String,
    pub max_balance: Option<String>,
    pub min_deposit_amount: Option<String>,
    pub max_deposit_amount: Option<String>,
    pub min_withdrawal_amount: Option<String>,
    pub max_withdrawal_amount: Option<String>,
    pub allow_overdraft: bool,
    pub overdraft_limit: Option<String>,
    pub nominal_annual_interest_rate: String,
    pub annual_interest_rate: String,
    pub min_balance_to_earn_interest: Option<String>,
    pub account_fee: Option<String>,
    pub maintenance_fee: Option<String>,
    pub withdrawal_fee: Option<String>,
    pub deposit_fee: Option<String>,
    pub inactive_account_fee: Option<String>,
    pub reject_if_insufficient_funds: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SavingsProduct {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: SavingsProductId,
        tenant_id: TenantId,
        name: String,
        short_name: String,
        description: Option<String>,
        currency_code: String,
        interest_rate: String,
        nominal_annual_interest_rate: String,
        annual_interest_rate: String,
    ) -> Result<Self, SavingsProductError> {
        if name.trim().is_empty() {
            return Err(SavingsProductError::EmptyName);
        }
        if short_name.trim().is_empty() {
            return Err(SavingsProductError::EmptyShortName);
        }
        if currency_code.trim().is_empty() {
            return Err(SavingsProductError::EmptyCurrencyCode);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            short_name,
            description,
            currency_code,
            currency_digits: 2,
            interest_rate,
            interest_compounding_period_type: 1,
            interest_posting_period_type: 1,
            interest_calculation_type: 1,
            interest_calculation_days_in_year_type: 1,
            min_required_opening_balance: "0".to_string(),
            max_balance: None,
            min_deposit_amount: None,
            max_deposit_amount: None,
            min_withdrawal_amount: None,
            max_withdrawal_amount: None,
            allow_overdraft: false,
            overdraft_limit: None,
            nominal_annual_interest_rate,
            annual_interest_rate,
            min_balance_to_earn_interest: None,
            account_fee: None,
            maintenance_fee: None,
            withdrawal_fee: None,
            deposit_fee: None,
            inactive_account_fee: None,
            reject_if_insufficient_funds: true,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SavingsProductError {
    #[error("savings product name must not be empty")]
    EmptyName,
    #[error("savings product short name must not be empty")]
    EmptyShortName,
    #[error("currency code must not be empty")]
    EmptyCurrencyCode,
    #[error("savings product not found: {0}")]
    NotFound(SavingsProductId),
    #[error("savings product {0} already exists")]
    AlreadyExists(SavingsProductId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl SavingsProductError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, SavingsProductError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for SavingsProductError {
    fn from(e: sqlx::Error) -> Self {
        SavingsProductError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for SavingsProductError {
    fn from(e: fineract_security::SecurityError) -> Self {
        SavingsProductError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn savings_product_creation_validates() {
        let result = SavingsProduct::new(
            SavingsProductId::new(1),
            TenantId::new(1),
            "".to_string(),
            "Test".to_string(),
            None,
            "USD".to_string(),
            "5".to_string(),
            "5".to_string(),
            "5".to_string(),
        );
        assert!(matches!(result, Err(SavingsProductError::EmptyName)));

        let result = SavingsProduct::new(
            SavingsProductId::new(1),
            TenantId::new(1),
            "Test".to_string(),
            "".to_string(),
            None,
            "USD".to_string(),
            "5".to_string(),
            "5".to_string(),
            "5".to_string(),
        );
        assert!(matches!(result, Err(SavingsProductError::EmptyShortName)));

        let result = SavingsProduct::new(
            SavingsProductId::new(1),
            TenantId::new(1),
            "Test".to_string(),
            "Test".to_string(),
            None,
            "".to_string(),
            "5".to_string(),
            "5".to_string(),
            "5".to_string(),
        );
        assert!(matches!(
            result,
            Err(SavingsProductError::EmptyCurrencyCode)
        ));

        let product = SavingsProduct::new(
            SavingsProductId::new(1),
            TenantId::new(1),
            "Test Savings".to_string(),
            "TS".to_string(),
            None,
            "USD".to_string(),
            "5".to_string(),
            "5".to_string(),
            "5".to_string(),
        )
        .unwrap();
        assert_eq!(product.name, "Test Savings");
    }
}
