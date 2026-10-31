//! Client domain models.
//! Clients, groups, and client-related entities.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

use fineract_core::ids::ClientId;
use fineract_core::ids::GroupId;
use fineract_core::ids::OfficeId;
use fineract_core::ids::StaffId;
use fineract_core::ids::TenantId;
use fineract_core::money::Currency;

/// Status of a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientStatus {
    Active,
    Inactive,
    Closed,
    Pending,
    Rejected,
}

/// Type of client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientType {
    Individual,
    Group,
}

/// A client (individual or group).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub tenant_id: TenantId,
    pub office_id: OfficeId,
    pub client_type: ClientType,
    pub status: ClientStatus,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub display_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub mobile_no: Option<String>,
    pub email: Option<String>,
    pub office_name: Option<String>,
    pub staff_id: Option<StaffId>,
    pub staff_name: Option<String>,
    pub external_id: Option<String>,
    pub active: bool,
    pub created_date: DateTime<Utc>,
    pub last_modified_date: Option<DateTime<Utc>>,
}

impl Client {
    pub fn new(
        id: ClientId,
        tenant_id: TenantId,
        office_id: OfficeId,
        client_type: ClientType,
        display_name: String,
        staff_id: Option<StaffId>,
    ) -> Result<Self, ClientError> {
        if display_name.trim().is_empty() {
            return Err(ClientError::EmptyDisplayName);
        }
        Ok(Self {
            id,
            tenant_id,
            office_id,
            client_type,
            status: ClientStatus::Pending,
            firstname: None,
            lastname: None,
            display_name,
            date_of_birth: None,
            gender: None,
            mobile_no: None,
            email: None,
            office_name: None,
            staff_id,
            staff_name: None,
            external_id: None,
            active: false,
            created_date: Utc::now(),
            last_modified_date: None,
        })
    }
}

/// A group (center) of clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub tenant_id: TenantId,
    pub office_id: OfficeId,
    pub name: String,
    pub display_name: String,
    pub status: GroupStatus,
    pub staff_id: Option<StaffId>,
    pub staff_name: Option<String>,
    pub external_id: Option<String>,
    pub active: bool,
    pub created_date: DateTime<Utc>,
    pub last_modified_date: Option<DateTime<Utc>>,
}

impl Group {
    pub fn new(
        id: GroupId,
        tenant_id: TenantId,
        office_id: OfficeId,
        display_name: String,
        staff_id: Option<StaffId>,
    ) -> Result<Self, ClientError> {
        if display_name.trim().is_empty() {
            return Err(ClientError::EmptyDisplayName);
        }
        Ok(Self {
            id,
            tenant_id,
            office_id,
            name: display_name.clone(),
            display_name,
            status: GroupStatus::Active,
            staff_id,
            staff_name: None,
            external_id: None,
            active: false,
            created_date: Utc::now(),
            last_modified_date: None,
        })
    }
}

/// Status of a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupStatus {
    Active,
    Inactive,
    Closed,
}

/// Error type for client domain operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClientError {
    #[error("display name must not be empty")]
    EmptyDisplayName,
    #[error("client not found: {0}")]
    NotFound(ClientId),
    #[error("client {0} already exists")]
    AlreadyExists(ClientId),
    #[error("group not found: {0}")]
    GroupNotFound(GroupId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl ClientError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, ClientError::TenantIsolationViolation)
    }
}

impl From<SqlxError> for ClientError {
    fn from(e: SqlxError) -> Self {
        ClientError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for ClientError {
    fn from(e: fineract_security::SecurityError) -> Self {
        ClientError::Internal {
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
    fn client_creation_validates_display_name() {
        let result = Client::new(
            ClientId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientType::Individual,
            "".to_string(),
            Some(StaffId::new(1)),
        );
        assert!(matches!(result, Err(ClientError::EmptyDisplayName)));

        let client = Client::new(
            ClientId::new(1),
            TenantId::new(1),
            OfficeId::new(1),
            ClientType::Individual,
            "John Doe".to_string(),
            Some(StaffId::new(1)),
        )
        .unwrap();
        assert_eq!(client.display_name, "John Doe");
        assert_eq!(client.client_type, ClientType::Individual);
    }

    #[test]
    fn client_error_tenant_isolation_check() {
        let err = ClientError::TenantIsolationViolation;
        assert!(err.is_tenant_isolation_violation());
        let err = ClientError::NotFound(ClientId::new(1));
        assert!(!err.is_tenant_isolation_violation());
    }
}
