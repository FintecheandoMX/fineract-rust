//! Progressive loan domain models.
//! Progressive loans release funds in phases based on milestones.

use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// Phase of a progressive loan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressiveLoanPhase {
    Application,
    Approved,
    Disbursed,
    Completed,
}

/// A progressive loan with phased disbursement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressiveLoan {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub client_id: ClientId,
    pub total_amount: rust_decimal::Decimal,
    pub currency: Currency,
    pub current_phase: ProgressiveLoanPhase,
    pub disbursed_amount: rust_decimal::Decimal,
    pub remaining_amount: rust_decimal::Decimal,
    pub interest_rate: rust_decimal::Decimal,
    pub active: bool,
}

impl ProgressiveLoan {
    pub fn new(
        id: LoanId,
        tenant_id: TenantId,
        client_id: ClientId,
        total_amount: rust_decimal::Decimal,
        currency: Currency,
        interest_rate: rust_decimal::Decimal,
    ) -> Result<Self, ProgressiveLoanError> {
        if total_amount.is_zero() || total_amount.is_sign_negative() {
            return Err(ProgressiveLoanError::InvalidPrincipal);
        }
        Ok(Self {
            id,
            tenant_id,
            client_id,
            total_amount,
            currency,
            current_phase: ProgressiveLoanPhase::Application,
            disbursed_amount: rust_decimal::Decimal::ZERO,
            remaining_amount: total_amount,
            interest_rate,
            active: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressiveLoanSchedule {
    pub phase: ProgressiveLoanPhase,
    pub amount: rust_decimal::Decimal,
    pub due_date: chrono::NaiveDate,
}

/// Error type for progressive loan domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProgressiveLoanError {
    #[error("progressive loan principal must be positive")]
    InvalidPrincipal,
    #[error("progressive loan not found: {0}")]
    NotFound(LoanId),
    #[error("progressive loan {0} already exists")]
    AlreadyExists(LoanId),
    #[error("cannot advance phase: already completed")]
    PhaseComplete,
    #[error("insufficient remaining amount")]
    InsufficientRemaining,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl ProgressiveLoanError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, ProgressiveLoanError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for ProgressiveLoanError {
    fn from(e: SqlxError) -> Self {
        ProgressiveLoanError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for ProgressiveLoanError {
    fn from(e: fineract_security::SecurityError) -> Self {
        ProgressiveLoanError::Internal {
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
    fn progressive_loan_creation_validates() {
        let result = ProgressiveLoan::new(
            LoanId::new(1),
            TenantId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::ZERO,
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
        );
        assert!(matches!(
            result,
            Err(ProgressiveLoanError::InvalidPrincipal)
        ));

        let loan = ProgressiveLoan::new(
            LoanId::new(1),
            TenantId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::new(500000, 2),
            Currency::USD,
            rust_decimal::Decimal::new(1000, 2),
        )
        .unwrap();
        assert_eq!(loan.total_amount, rust_decimal::Decimal::new(500000, 2));
        assert_eq!(loan.current_phase, ProgressiveLoanPhase::Application);
        assert_eq!(loan.disbursed_amount, rust_decimal::Decimal::ZERO);
    }
}
