//! Working capital loan domain models.
//! Revolving working capital loans with dynamic drawdown and repayment.

use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// Status of a working capital loan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkingCapitalLoanStatus {
    Active,
    Suspended,
    Closed,
}

/// A working capital loan with revolving credit line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingCapitalLoan {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub client_id: ClientId,
    pub credit_limit: rust_decimal::Decimal,
    pub currency: Currency,
    pub outstanding_balance: rust_decimal::Decimal,
    pub available_balance: rust_decimal::Decimal,
    pub interest_rate: rust_decimal::Decimal,
    pub status: WorkingCapitalLoanStatus,
    pub active: bool,
}

impl WorkingCapitalLoan {
    pub fn new(
        id: LoanId,
        tenant_id: TenantId,
        client_id: ClientId,
        credit_limit: rust_decimal::Decimal,
        currency: Currency,
        interest_rate: rust_decimal::Decimal,
    ) -> Result<Self, WorkingCapitalLoanError> {
        if credit_limit.is_zero() || credit_limit.is_sign_negative() {
            return Err(WorkingCapitalLoanError::InvalidCreditLimit);
        }
        Ok(Self {
            id,
            tenant_id,
            client_id,
            credit_limit,
            currency,
            outstanding_balance: rust_decimal::Decimal::ZERO,
            available_balance: credit_limit,
            interest_rate,
            status: WorkingCapitalLoanStatus::Active,
            active: true,
        })
    }

    pub fn can_draw(&self, amount: rust_decimal::Decimal) -> bool {
        self.status == WorkingCapitalLoanStatus::Active
            && self.active
            && amount > rust_decimal::Decimal::ZERO
            && self.available_balance >= amount
    }
}

/// A drawdown on a working capital loan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingCapitalDrawdown {
    pub id: i64,
    pub loan_id: LoanId,
    pub tenant_id: TenantId,
    pub amount: rust_decimal::Decimal,
    pub drawdown_date: chrono::DateTime<chrono::Utc>,
    pub active: bool,
}

/// Error type for working capital loan domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WorkingCapitalLoanError {
    #[error("credit limit must be positive")]
    InvalidCreditLimit,
    #[error("working capital loan not found: {0}")]
    NotFound(LoanId),
    #[error("working capital loan {0} already exists")]
    AlreadyExists(LoanId),
    #[error("insufficient available balance")]
    InsufficientAvailableBalance,
    #[error("loan is not active")]
    NotActive,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl WorkingCapitalLoanError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, WorkingCapitalLoanError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for WorkingCapitalLoanError {
    fn from(e: SqlxError) -> Self {
        WorkingCapitalLoanError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for WorkingCapitalLoanError {
    fn from(e: fineract_security::SecurityError) -> Self {
        WorkingCapitalLoanError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;
    use fineract_core::money::Currency;

    #[test]
    fn working_capital_loan_creation_validates() {
        let result = WorkingCapitalLoan::new(
            LoanId::new(1),
            TenantId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::ZERO,
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
        );
        assert!(matches!(
            result,
            Err(WorkingCapitalLoanError::InvalidCreditLimit)
        ));

        let loan = WorkingCapitalLoan::new(
            LoanId::new(1),
            TenantId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::new(200000, 2),
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
        )
        .unwrap();
        assert_eq!(loan.credit_limit, rust_decimal::Decimal::new(200000, 2));
        assert_eq!(
            loan.available_balance,
            rust_decimal::Decimal::new(200000, 2)
        );
        assert!(loan.can_draw(rust_decimal::Decimal::new(50000, 2)));
    }
}
