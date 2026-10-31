//! Savings domain models.
//! Savings accounts, transaction entries, and account types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::SavingsAccountId;
use fineract_core::ids::SavingsTransactionId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// Type of savings account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavingsAccountType {
    /// Demand / current account.
    Demand,
    /// Savings account.
    Savings,
    /// Fixed deposit / term deposit.
    FixedDeposit,
}

/// Status of a savings account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavingsAccountStatus {
    Active,
    Frozen,
    Closed,
    PendingApproval,
}

/// A savings account aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavingsAccount {
    pub id: SavingsAccountId,
    pub tenant_id: TenantId,
    pub account_no: String,
    pub account_type: SavingsAccountType,
    pub status: SavingsAccountStatus,
    pub currency: Currency,
    pub balance: rust_decimal::Decimal,
    pub interest_rate: Option<rust_decimal::Decimal>,
    pub opened_date: DateTime<Utc>,
    pub closed_date: Option<DateTime<Utc>>,
    pub active: bool,
}

impl SavingsAccount {
    pub fn new(
        id: SavingsAccountId,
        tenant_id: TenantId,
        account_no: String,
        account_type: SavingsAccountType,
        currency: Currency,
        opened_date: DateTime<Utc>,
    ) -> Result<Self, SavingsError> {
        if account_no.trim().is_empty() {
            return Err(SavingsError::EmptyAccountNo);
        }
        Ok(Self {
            id,
            tenant_id,
            account_no,
            account_type,
            status: SavingsAccountStatus::PendingApproval,
            currency,
            balance: rust_decimal::Decimal::ZERO,
            interest_rate: None,
            opened_date,
            closed_date: None,
            active: false,
        })
    }

    /// Returns true if the account can accept deposits/withdrawals.
    pub fn is_operational(&self) -> bool {
        self.status == SavingsAccountStatus::Active && self.active
    }
}

/// A transaction entry on a savings account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavingsTransaction {
    pub id: SavingsTransactionId,
    pub savings_account_id: SavingsAccountId,
    pub tenant_id: TenantId,
    pub amount: rust_decimal::Decimal,
    pub transaction_type: TransactionType,
    pub transaction_date: DateTime<Utc>,
    pub reference: Option<String>,
    pub active: bool,
}

/// Type of savings transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Interest,
    Reversal,
    Fee,
}

/// Error type for savings domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SavingsError {
    #[error("account number must not be empty")]
    EmptyAccountNo,
    #[error("savings account not found: {0}")]
    NotFound(SavingsAccountId),
    #[error("savings account {0} already exists")]
    AlreadyExists(SavingsAccountId),
    #[error("insufficient funds")]
    InsufficientFunds,
    #[error("account is not operational")]
    NotOperational,
    #[error("amount must be positive")]
    NonPositiveAmount,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl SavingsError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, SavingsError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for SavingsError {
    fn from(e: SqlxError) -> Self {
        SavingsError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for SavingsError {
    fn from(e: fineract_security::SecurityError) -> Self {
        SavingsError::Internal {
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
    fn savings_account_creation_validates() {
        let result = SavingsAccount::new(
            SavingsAccountId::new(1),
            TenantId::new(1),
            "".to_string(),
            SavingsAccountType::Savings,
            Currency::USD,
            Utc::now(),
        );
        assert!(matches!(result, Err(SavingsError::EmptyAccountNo)));

        let account = SavingsAccount::new(
            SavingsAccountId::new(1),
            TenantId::new(1),
            "SA-001234".to_string(),
            SavingsAccountType::Savings,
            Currency::USD,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(account.account_no, "SA-001234");
        assert!(!account.is_operational()); // PendingApproval
    }

    #[test]
    fn is_operational_when_active() {
        let mut account = SavingsAccount {
            id: SavingsAccountId::new(1),
            tenant_id: TenantId::new(1),
            account_no: "SA-001".to_string(),
            account_type: SavingsAccountType::Savings,
            status: SavingsAccountStatus::Active,
            currency: Currency::USD,
            balance: rust_decimal::Decimal::ZERO,
            interest_rate: None,
            opened_date: Utc::now(),
            closed_date: None,
            active: true,
        };
        assert!(account.is_operational());

        account.status = SavingsAccountStatus::Frozen;
        assert!(!account.is_operational());
    }

    #[test]
    fn savings_error_tenant_isolation_check() {
        let err = SavingsError::TenantIsolationViolation;
        assert!(err.is_tenant_isolation_violation());
        let err = SavingsError::NotFound(SavingsAccountId::new(1));
        assert!(!err.is_tenant_isolation_violation());
    }
}
