//! Staff domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;

/// A staff member (loan officer, teller, manager, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Staff {
    pub id: StaffId,
    pub tenant_id: TenantId,
    pub office_id: Option<OfficeId>,
    pub firstname: String,
    pub lastname: String,
    pub username: String,
    pub email: Option<String>,
    pub mobile_no: Option<String>,
    pub is_loan_officer: bool,
    pub is_active: bool,
    pub password_hash: Option<String>,
    pub external_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Staff {
    pub fn new(
        id: StaffId,
        tenant_id: TenantId,
        firstname: String,
        lastname: String,
        username: String,
        office_id: Option<OfficeId>,
    ) -> Result<Self, StaffError> {
        if firstname.trim().is_empty() {
            return Err(StaffError::EmptyFirstname);
        }
        if lastname.trim().is_empty() {
            return Err(StaffError::EmptyLastname);
        }
        if username.trim().is_empty() {
            return Err(StaffError::EmptyUsername);
        }
        Ok(Self {
            id,
            tenant_id,
            office_id,
            firstname,
            lastname,
            username,
            email: None,
            mobile_no: None,
            is_loan_officer: false,
            is_active: true,
            password_hash: None,
            external_id: None,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.firstname, self.lastname)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StaffError {
    #[error("staff firstname must not be empty")]
    EmptyFirstname,
    #[error("staff lastname must not be empty")]
    EmptyLastname,
    #[error("staff username must not be empty")]
    EmptyUsername,
    #[error("staff not found: {0}")]
    NotFound(StaffId),
    #[error("staff {0} already exists")]
    AlreadyExists(StaffId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl StaffError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, StaffError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for StaffError {
    fn from(e: sqlx::Error) -> Self {
        StaffError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for StaffError {
    fn from(e: fineract_security::SecurityError) -> Self {
        StaffError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn staff_creation_validates() {
        let result = Staff::new(
            StaffId::new(1),
            TenantId::new(1),
            "".to_string(),
            "Doe".to_string(),
            "jdoe".to_string(),
            None,
        );
        assert!(matches!(result, Err(StaffError::EmptyFirstname)));

        let result = Staff::new(
            StaffId::new(1),
            TenantId::new(1),
            "John".to_string(),
            "".to_string(),
            "jdoe".to_string(),
            None,
        );
        assert!(matches!(result, Err(StaffError::EmptyLastname)));

        let result = Staff::new(
            StaffId::new(1),
            TenantId::new(1),
            "John".to_string(),
            "Doe".to_string(),
            "".to_string(),
            None,
        );
        assert!(matches!(result, Err(StaffError::EmptyUsername)));

        let staff = Staff::new(
            StaffId::new(1),
            TenantId::new(1),
            "John".to_string(),
            "Doe".to_string(),
            "jdoe".to_string(),
            Some(OfficeId::new(1)),
        )
        .unwrap();
        assert_eq!(staff.full_name(), "John Doe");
        assert_eq!(staff.username, "jdoe");
    }
}
