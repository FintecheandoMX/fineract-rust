//! Loan domain models.
//! Loan aggregate, transactions, schedule, and product definitions.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::ClientId;
use fineract_core::ids::CollateralId;
use fineract_core::ids::FundId;
use fineract_core::ids::LoanId;
use fineract_core::ids::LoanScheduleId;
use fineract_core::ids::LoanTransactionId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// Type of loan product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanType {
    Individual,
    Group,
}

/// Status of a loan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanStatus {
    PendingApproval,
    Approved,
    Active,
    Overpaid,
    Closed,
    Rejected,
    WrittenOff,
}

/// A loan aggregate root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Loan {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub office_id: OfficeId,
    pub client_id: ClientId,
    pub loan_officer_id: Option<StaffId>,
    pub loan_type: LoanType,
    pub product_id: i64,
    pub principal: rust_decimal::Decimal,
    pub currency: Currency,
    pub interest_rate: rust_decimal::Decimal,
    pub term_count: i32,
    pub term_period: LoanTermPeriod,
    pub repayment_count: i32,
    pub repayment_every: i32,
    pub disbursement_date: NaiveDate,
    pub status: LoanStatus,
    pub external_id: Option<String>,
    pub active: bool,
}

impl Loan {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: LoanId,
        tenant_id: TenantId,
        office_id: OfficeId,
        client_id: ClientId,
        loan_type: LoanType,
        principal: rust_decimal::Decimal,
        currency: Currency,
        interest_rate: rust_decimal::Decimal,
        term_count: i32,
        term_period: LoanTermPeriod,
        disbursement_date: NaiveDate,
    ) -> Result<Self, LoanError> {
        if principal.is_zero() || principal.is_sign_negative() {
            return Err(LoanError::InvalidPrincipal);
        }
        if interest_rate.is_sign_negative() {
            return Err(LoanError::InvalidInterestRate);
        }
        Ok(Self {
            id,
            tenant_id,
            office_id,
            client_id,
            loan_officer_id: None,
            loan_type,
            product_id: 0,
            principal,
            currency,
            interest_rate,
            term_count,
            term_period,
            repayment_count: term_count,
            repayment_every: 1,
            disbursement_date,
            status: LoanStatus::PendingApproval,
            external_id: None,
            active: true,
        })
    }

    pub fn is_active(&self) -> bool {
        self.status == LoanStatus::Active && self.active
    }
}

/// Term period for loan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanTermPeriod {
    Days,
    Weeks,
    Months,
    Years,
}

/// A repayment schedule installment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanSchedule {
    pub id: LoanScheduleId,
    pub loan_id: LoanId,
    pub tenant_id: TenantId,
    pub installment_number: i32,
    pub due_date: NaiveDate,
    pub principal_due: rust_decimal::Decimal,
    pub interest_due: rust_decimal::Decimal,
    pub total_due: rust_decimal::Decimal,
    pub principal_paid: rust_decimal::Decimal,
    pub interest_paid: rust_decimal::Decimal,
    pub principal_outstanding: rust_decimal::Decimal,
    pub interest_outstanding: rust_decimal::Decimal,
    pub penalty: rust_decimal::Decimal,
    pub is_paid: bool,
}

/// A transaction on a loan (disbursement, repayment, fee, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanTransaction {
    pub id: LoanTransactionId,
    pub loan_id: LoanId,
    pub tenant_id: TenantId,
    pub amount: rust_decimal::Decimal,
    pub transaction_type: LoanTransactionType,
    pub transaction_date: DateTime<Utc>,
    pub reference: Option<String>,
    pub active: bool,
}

/// Type of loan transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanTransactionType {
    Disbursement,
    Repayment,
    Fee,
    Interest,
    Reversal,
    WriteOff,
}

/// Collateral securing a loan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanCollateral {
    pub id: CollateralId,
    pub loan_id: LoanId,
    pub tenant_id: TenantId,
    pub collateral_type: String,
    pub value: rust_decimal::Decimal,
    pub currency: Currency,
    pub active: bool,
}

/// Error type for loan domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LoanError {
    #[error("loan principal must be positive")]
    InvalidPrincipal,
    #[error("interest rate must not be negative")]
    InvalidInterestRate,
    #[error("loan not found: {0}")]
    NotFound(LoanId),
    #[error("loan {0} already exists")]
    AlreadyExists(LoanId),
    #[error("insufficient principal outstanding")]
    InsufficientOutstanding,
    #[error("loan is not active")]
    NotActive,
    #[error("cannot modify approved loan")]
    CannotModify,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl LoanError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, LoanError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for LoanError {
    fn from(e: SqlxError) -> Self {
        LoanError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for LoanError {
    fn from(e: fineract_security::SecurityError) -> Self {
        LoanError::Internal {
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
    fn loan_creation_validates() {
        let result = Loan::new(
            LoanId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            LoanType::Individual,
            rust_decimal::Decimal::ZERO,
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
            12,
            LoanTermPeriod::Months,
            chrono::Utc::now().date_naive(),
        );
        assert!(matches!(result, Err(LoanError::InvalidPrincipal)));

        let result = Loan::new(
            LoanId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            LoanType::Individual,
            rust_decimal::Decimal::new(100000, 2),
            Currency::USD,
            rust_decimal::Decimal::new(-500, 2),
            12,
            LoanTermPeriod::Months,
            chrono::Utc::now().date_naive(),
        );
        assert!(matches!(result, Err(LoanError::InvalidInterestRate)));

        let loan = Loan::new(
            LoanId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            LoanType::Individual,
            rust_decimal::Decimal::new(100000, 2),
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
            12,
            LoanTermPeriod::Months,
            chrono::Utc::now().date_naive(),
        )
        .unwrap();
        assert_eq!(loan.principal, rust_decimal::Decimal::new(100000, 2));
        assert_eq!(loan.status, LoanStatus::PendingApproval);
        assert!(!loan.is_active());
    }

    #[test]
    fn loan_is_active_when_status_active() {
        let mut loan = Loan {
            id: LoanId::new(1),
            tenant_id: TenantId::new(1),
            office_id: OfficeId::new(1),
            client_id: ClientId::new(1),
            loan_officer_id: None,
            loan_type: LoanType::Individual,
            product_id: 0,
            principal: rust_decimal::Decimal::new(100000, 2),
            currency: Currency::USD,
            interest_rate: rust_decimal::Decimal::new(1000, 2),
            term_count: 12,
            term_period: LoanTermPeriod::Months,
            repayment_count: 12,
            repayment_every: 1,
            disbursement_date: chrono::Utc::now().date_naive(),
            status: LoanStatus::Active,
            external_id: None,
            active: true,
        };
        assert!(loan.is_active());

        loan.status = LoanStatus::Closed;
        assert!(!loan.is_active());
    }

    #[test]
    fn loan_error_tenant_isolation_check() {
        let err = LoanError::TenantIsolationViolation;
        assert!(err.is_tenant_isolation_violation());
        let err = LoanError::NotFound(LoanId::new(1));
        assert!(!err.is_tenant_isolation_violation());
    }
}
