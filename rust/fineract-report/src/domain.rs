//! Report domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::ReportId;
use fineract_core::ids::TenantId;

/// Type of report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportType {
    Portfolio,
    Savings,
    Loan,
    Client,
    Accounting,
    Delinquency,
    Disbursement,
    Repayment,
    Aging,
    Custom,
}

/// Report generation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportStatus {
    Pending,
    Generating,
    Completed,
    Failed,
    Cancelled,
}

/// Output format for reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Pdf,
    Excel,
    Csv,
    Html,
    Json,
}

/// A report definition and execution record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub id: ReportId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub report_type: ReportType,
    pub report_format: ReportFormat,
    pub parameters: Option<serde_json::Value>,
    pub status: ReportStatus,
    pub file_path: Option<String>,
    pub generated_by: Option<i64>,
    pub generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Report {
    pub fn new(
        id: ReportId,
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
        report_type: ReportType,
        report_format: ReportFormat,
        parameters: Option<serde_json::Value>,
    ) -> Result<Self, ReportError> {
        if name.trim().is_empty() {
            return Err(ReportError::EmptyName);
        }
        Ok(Self {
            id,
            tenant_id,
            name,
            description,
            report_type,
            report_format,
            parameters,
            status: ReportStatus::Pending,
            file_path: None,
            generated_by: None,
            generated_at: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReportError {
    #[error("report name must not be empty")]
    EmptyName,
    #[error("report not found: {0}")]
    NotFound(ReportId),
    #[error("report {0} already exists")]
    AlreadyExists(ReportId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl ReportError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, ReportError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for ReportError {
    fn from(e: sqlx::Error) -> Self {
        ReportError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for ReportError {
    fn from(e: fineract_security::SecurityError) -> Self {
        ReportError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn report_creation_validates() {
        let result = Report::new(
            ReportId::new(1),
            TenantId::new(1),
            "".to_string(),
            None,
            ReportType::Portfolio,
            ReportFormat::Pdf,
            None,
        );
        assert!(matches!(result, Err(ReportError::EmptyName)));

        let report = Report::new(
            ReportId::new(1),
            TenantId::new(1),
            "Portfolio Report".to_string(),
            None,
            ReportType::Portfolio,
            ReportFormat::Pdf,
            None,
        )
        .unwrap();
        assert_eq!(report.name, "Portfolio Report");
        assert_eq!(report.status, ReportStatus::Pending);
    }
}
