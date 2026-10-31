//! Domain models for the branch / office aggregate.
//!
//! Offices form a hierarchical tree within a tenant. Each office has a name,
//! optional external ID, opening date, timezone, country, currency, and an
//! optional parent office.

use chrono::NaiveDate;
use fineract_security::SecurityError;
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::OfficeId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// An office / branch in the organisational hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Office {
    pub id: OfficeId,
    pub tenant_id: TenantId,
    pub name: String,
    pub external_id: Option<String>,
    pub opening_date: Option<NaiveDate>,
    pub timezone: Option<String>,
    pub country: Option<String>,
    pub currency: Currency,
    pub parent_office_id: Option<OfficeId>,
    pub active: bool,
}

impl Office {
    /// Create a new office. Validates that name is non-empty.
    pub fn new(
        id: OfficeId,
        tenant_id: TenantId,
        name: String,
        currency: Currency,
    ) -> Result<Self, OfficeError> {
        if name.trim().is_empty() {
            return Err(OfficeError::EmptyName);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            external_id: None,
            opening_date: None,
            timezone: None,
            country: None,
            currency,
            parent_office_id: None,
            active: true,
        })
    }

    /// Returns true if this office is a root office (no parent).
    pub fn is_root(&self) -> bool {
        self.parent_office_id.is_none()
    }
}

/// Error type for office domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OfficeError {
    #[error("office name must not be empty")]
    EmptyName,
    #[error("office not found: {0}")]
    NotFound(OfficeId),
    #[error("office {0} already exists")]
    AlreadyExists(OfficeId),
    #[error("cannot set office {0} as its own parent")]
    SelfParent(OfficeId),
    #[error("parent office {0} does not exist in this tenant")]
    ParentNotFound(OfficeId),
    #[error("parent office {0} belongs to a different tenant")]
    ParentTenantMismatch(OfficeId),
    #[error("creating a cycle in the office hierarchy")]
    HierarchyCycle,
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl OfficeError {
    /// Returns true if this error represents a tenant isolation violation (IDOR attempt).
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, OfficeError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for OfficeError {
    fn from(e: SqlxError) -> Self {
        OfficeError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<SecurityError> for OfficeError {
    fn from(e: SecurityError) -> Self {
        OfficeError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn office_creation_validates_name() {
        let result = Office::new(
            OfficeId::new(1),
            TenantId::new(1),
            "".to_string(),
            fineract_core::money::Currency::USD,
        );
        assert!(matches!(result, Err(OfficeError::EmptyName)));

        let office = Office::new(
            OfficeId::new(1),
            TenantId::new(1),
            "Main Branch".to_string(),
            fineract_core::money::Currency::USD,
        )
        .unwrap();
        assert_eq!(office.name, "Main Branch");
        assert!(office.is_root());
    }

    #[test]
    fn office_is_root_when_no_parent() {
        let office = Office {
            id: OfficeId::new(1),
            tenant_id: TenantId::new(1),
            name: "Root".to_string(),
            external_id: None,
            opening_date: None,
            timezone: None,
            country: None,
            currency: fineract_core::money::Currency::USD,
            parent_office_id: None,
            active: true,
        };
        assert!(office.is_root());

        let office = Office {
            id: OfficeId::new(2),
            tenant_id: TenantId::new(1),
            name: "Child".to_string(),
            external_id: None,
            opening_date: None,
            timezone: None,
            country: None,
            currency: fineract_core::money::Currency::USD,
            parent_office_id: Some(OfficeId::new(1)),
            active: true,
        };
        assert!(!office.is_root());
    }
}
