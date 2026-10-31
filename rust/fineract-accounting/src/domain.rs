//! Accounting domain models.
//! GL accounts, journal entries, and accounting periods.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::AccountId;
use fineract_core::ids::AccountingPeriodId;
use fineract_core::ids::JournalEntryId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// Type of GL account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

/// Status of a GL account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Closed,
}

/// A general ledger account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub tenant_id: TenantId,
    pub account_code: String,
    pub account_name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    pub status: AccountStatus,
    pub balance: rust_decimal::Decimal,
    pub active: bool,
}

impl Account {
    pub fn new(
        id: AccountId,
        tenant_id: TenantId,
        account_code: String,
        account_name: String,
        account_type: AccountType,
        currency: Currency,
    ) -> Result<Self, AccountingError> {
        if account_code.trim().is_empty() {
            return Err(AccountingError::EmptyAccountCode);
        }
        if account_name.trim().is_empty() {
            return Err(AccountingError::EmptyAccountName);
        }
        Ok(Self {
            id,
            tenant_id,
            account_code,
            account_name,
            account_type,
            currency,
            status: AccountStatus::Active,
            balance: rust_decimal::Decimal::ZERO,
            active: true,
        })
    }
}

/// A journal entry (double-entry bookkeeping).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: JournalEntryId,
    pub tenant_id: TenantId,
    pub entry_date: DateTime<Utc>,
    pub reference: String,
    pub total_debit: rust_decimal::Decimal,
    pub total_credit: rust_decimal::Decimal,
    pub active: bool,
}

/// Direction of a journal entry line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryLineType {
    Debit,
    Credit,
}

/// A line item within a journal entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntryLine {
    pub id: i64,
    pub journal_entry_id: JournalEntryId,
    pub account_id: AccountId,
    pub line_type: EntryLineType,
    pub amount: rust_decimal::Decimal,
    pub active: bool,
}

/// An accounting period (open/closed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountingPeriod {
    pub id: AccountingPeriodId,
    pub tenant_id: TenantId,
    pub period_name: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub is_closed: bool,
}

/// Error type for accounting domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AccountingError {
    #[error("account code must not be empty")]
    EmptyAccountCode,
    #[error("account name must not be empty")]
    EmptyAccountName,
    #[error("account not found: {0}")]
    NotFound(AccountId),
    #[error("account {0} already exists")]
    AlreadyExists(AccountId),
    #[error("journal entry not found: {0}")]
    JournalEntryNotFound(JournalEntryId),
    #[error("journal entry is not balanced")]
    UnbalancedEntry,
    #[error("accounting period is closed")]
    PeriodClosed,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl AccountingError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, AccountingError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for AccountingError {
    fn from(e: SqlxError) -> Self {
        AccountingError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for AccountingError {
    fn from(e: fineract_security::SecurityError) -> Self {
        AccountingError::Internal {
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
    fn account_creation_validates() {
        let result = Account::new(
            AccountId::new(1),
            TenantId::new(1),
            "".to_string(),
            "Cash".to_string(),
            AccountType::Asset,
            Currency::USD,
        );
        assert!(matches!(result, Err(AccountingError::EmptyAccountCode)));

        let result = Account::new(
            AccountId::new(1),
            TenantId::new(1),
            "1000".to_string(),
            "".to_string(),
            AccountType::Asset,
            Currency::USD,
        );
        assert!(matches!(result, Err(AccountingError::EmptyAccountName)));

        let account = Account::new(
            AccountId::new(1),
            TenantId::new(1),
            "1000".to_string(),
            "Cash on Hand".to_string(),
            AccountType::Asset,
            Currency::USD,
        )
        .unwrap();
        assert_eq!(account.account_code, "1000");
        assert_eq!(account.account_name, "Cash on Hand");
        assert_eq!(account.account_type, AccountType::Asset);
        assert!(account.active);
    }

    #[test]
    fn journal_entry_balance_check() {
        let entry = JournalEntry {
            id: JournalEntryId::new(1),
            tenant_id: TenantId::new(1),
            entry_date: Utc::now(),
            reference: "JE-001".to_string(),
            total_debit: rust_decimal::Decimal::new(1000, 2),
            total_credit: rust_decimal::Decimal::new(1000, 2),
            active: true,
        };
        assert_eq!(entry.total_debit, entry.total_credit);
    }

    #[test]
    fn accounting_error_tenant_isolation_check() {
        let err = AccountingError::TenantIsolationViolation;
        assert!(err.is_tenant_isolation_violation());
        let err = AccountingError::NotFound(AccountId::new(1));
        assert!(!err.is_tenant_isolation_violation());
    }
}
