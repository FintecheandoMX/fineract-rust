//! Loan origination domain models.
//! Loan application, review, and approval workflow.

use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::ClientId;
use fineract_core::ids::LoanId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// Status of a loan application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanApplicationStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Withdrawn,
}

/// A loan application submitted by a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoanApplication {
    pub id: LoanId,
    pub tenant_id: TenantId,
    pub office_id: OfficeId,
    pub client_id: ClientId,
    pub requested_amount: rust_decimal::Decimal,
    pub currency: Currency,
    pub purpose: String,
    pub status: LoanApplicationStatus,
    pub submitted_by: StaffId,
    pub reviewed_by: Option<StaffId>,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejection_reason: Option<String>,
    pub active: bool,
}

impl LoanApplication {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: LoanId,
        tenant_id: TenantId,
        office_id: OfficeId,
        client_id: ClientId,
        requested_amount: rust_decimal::Decimal,
        currency: Currency,
        purpose: String,
        submitted_by: StaffId,
    ) -> Result<Self, LoanOriginationError> {
        if requested_amount.is_zero() || requested_amount.is_sign_negative() {
            return Err(LoanOriginationError::InvalidAmount);
        }
        if purpose.trim().is_empty() {
            return Err(LoanOriginationError::EmptyPurpose);
        }
        Ok(Self {
            id,
            tenant_id,
            office_id,
            client_id,
            requested_amount,
            currency,
            purpose,
            status: LoanApplicationStatus::Submitted,
            submitted_by,
            reviewed_by: None,
            submitted_at: chrono::Utc::now(),
            reviewed_at: None,
            rejection_reason: None,
            active: true,
        })
    }
}

/// Error type for loan origination domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LoanOriginationError {
    #[error("loan application amount must be positive")]
    InvalidAmount,
    #[error("loan application purpose must not be empty")]
    EmptyPurpose,
    #[error("loan application not found: {0}")]
    NotFound(LoanId),
    #[error("loan application {0} already exists")]
    AlreadyExists(LoanId),
    #[error("cannot modify submitted application")]
    CannotModify,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl LoanOriginationError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, LoanOriginationError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for LoanOriginationError {
    fn from(e: SqlxError) -> Self {
        LoanOriginationError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for LoanOriginationError {
    fn from(e: fineract_security::SecurityError) -> Self {
        LoanOriginationError::Internal {
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
    fn loan_application_creation_validates() {
        let result = LoanApplication::new(
            LoanId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::ZERO,
            Currency::USD,
            "Working capital".to_string(),
            StaffId::new(1),
        );
        assert!(matches!(result, Err(LoanOriginationError::InvalidAmount)));

        let result = LoanApplication::new(
            LoanId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::new(50000, 2),
            Currency::USD,
            "".to_string(),
            StaffId::new(1),
        );
        assert!(matches!(result, Err(LoanOriginationError::EmptyPurpose)));

        let app = LoanApplication::new(
            LoanId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientId::new(1),
            rust_decimal::Decimal::new(50000, 2),
            Currency::USD,
            "Equipment purchase".to_string(),
            StaffId::new(1),
        )
        .unwrap();
        assert_eq!(app.requested_amount, rust_decimal::Decimal::new(50000, 2));
        assert_eq!(app.status, LoanApplicationStatus::Submitted);
    }
}
