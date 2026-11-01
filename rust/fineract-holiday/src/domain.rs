//! Holiday domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::HolidayId;
use fineract_core::ids::TenantId;

/// A holiday or non-working day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holiday {
    pub id: HolidayId,
    pub tenant_id: TenantId,
    pub name: String,
    pub holiday_date: chrono::NaiveDate,
    pub is_recurring: bool,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Holiday {
    pub fn new(
        id: HolidayId,
        tenant_id: TenantId,
        name: String,
        holiday_date: chrono::NaiveDate,
        is_recurring: bool,
    ) -> Result<Self, HolidayError> {
        if name.trim().is_empty() {
            return Err(HolidayError::EmptyName);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            holiday_date,
            is_recurring,
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HolidayError {
    #[error("holiday name must not be empty")]
    EmptyName,
    #[error("holiday not found: {0}")]
    NotFound(HolidayId),
    #[error("holiday {0} already exists")]
    AlreadyExists(HolidayId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl HolidayError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, HolidayError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for HolidayError {
    fn from(e: sqlx::Error) -> Self {
        HolidayError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for HolidayError {
    fn from(e: fineract_security::SecurityError) -> Self {
        HolidayError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn holiday_creation_validates() {
        let result = Holiday::new(
            HolidayId::new(1),
            TenantId::new(1),
            "".to_string(),
            chrono::NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
            false,
        );
        assert!(matches!(result, Err(HolidayError::EmptyName)));

        let holiday = Holiday::new(
            HolidayId::new(1),
            TenantId::new(1),
            "Christmas".to_string(),
            chrono::NaiveDate::from_ymd_opt(2024, 12, 25).unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(holiday.name, "Christmas");
        assert_eq!(holiday.is_recurring, true);
    }
}
