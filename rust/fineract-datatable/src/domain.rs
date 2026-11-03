//! Datatable domain models.

use serde::{Deserialize, Serialize};

use fineract_core::ids::DatatableId;
use fineract_core::ids::TenantId;

/// A datatable definition (metadata for dynamic tables).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Datatable {
    pub id: DatatableId,
    pub tenant_id: TenantId,
    pub registered_table_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub multi_tenant: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A datatable column definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatatableColumn {
    pub id: DatatableColumnId,
    pub tenant_id: TenantId,
    pub datatable_id: DatatableId,
    pub column_name: String,
    pub display_name: String,
    pub column_type: String,
    pub column_length: Option<i32>,
    pub column_precision: Option<i32>,
    pub column_scale: Option<i32>,
    pub is_mandatory: bool,
    pub is_unique: bool,
    pub is_foreign_key: bool,
    pub foreign_key_table: Option<String>,
    pub foreign_key_column: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A datatable view (saved query/filter on a datatable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatatableView {
    pub id: DatatableViewId,
    pub tenant_id: TenantId,
    pub datatable_id: DatatableId,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<String>,
    pub filter_criteria: Option<serde_json::Value>,
    pub sort_criteria: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Identifier for a datatable column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DatatableColumnId(pub i64);

impl DatatableColumnId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    pub fn value(self) -> i64 {
        self.0
    }
}

impl From<DatatableColumnId> for i64 {
    fn from(id: DatatableColumnId) -> i64 {
        id.0
    }
}

impl std::fmt::Display for DatatableColumnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for a datatable view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DatatableViewId(pub i64);

impl DatatableViewId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    pub fn value(self) -> i64 {
        self.0
    }
}

impl From<DatatableViewId> for i64 {
    fn from(id: DatatableViewId) -> i64 {
        id.0
    }
}

impl std::fmt::Display for DatatableViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Datatable {
    pub fn new(
        id: DatatableId,
        tenant_id: TenantId,
        registered_table_name: String,
        display_name: String,
        description: Option<String>,
        multi_tenant: bool,
    ) -> Result<Self, DatatableError> {
        if registered_table_name.trim().is_empty() {
            return Err(DatatableError::EmptyTableName);
        }
        if display_name.trim().is_empty() {
            return Err(DatatableError::EmptyDisplayName);
        }
        Ok(Self {
            id,
            tenant_id,
            registered_table_name,
            display_name,
            description,
            multi_tenant,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

impl DatatableColumn {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: DatatableColumnId,
        tenant_id: TenantId,
        datatable_id: DatatableId,
        column_name: String,
        display_name: String,
        column_type: String,
        column_length: Option<i32>,
        column_precision: Option<i32>,
        column_scale: Option<i32>,
        is_mandatory: bool,
        is_unique: bool,
        is_foreign_key: bool,
        foreign_key_table: Option<String>,
        foreign_key_column: Option<String>,
    ) -> Result<Self, DatatableError> {
        if column_name.trim().is_empty() {
            return Err(DatatableError::EmptyColumnName);
        }
        if display_name.trim().is_empty() {
            return Err(DatatableError::EmptyDisplayName);
        }
        if column_type.trim().is_empty() {
            return Err(DatatableError::EmptyColumnType);
        }
        Ok(Self {
            id,
            tenant_id,
            datatable_id,
            column_name,
            display_name,
            column_type,
            column_length,
            column_precision,
            column_scale,
            is_mandatory,
            is_unique,
            is_foreign_key,
            foreign_key_table,
            foreign_key_column,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

impl DatatableView {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: DatatableViewId,
        tenant_id: TenantId,
        datatable_id: DatatableId,
        name: String,
        description: Option<String>,
        columns: Vec<String>,
        filter_criteria: Option<serde_json::Value>,
        sort_criteria: Option<serde_json::Value>,
    ) -> Result<Self, DatatableError> {
        if name.trim().is_empty() {
            return Err(DatatableError::EmptyName);
        }
        if columns.is_empty() {
            return Err(DatatableError::EmptyColumns);
        }
        Ok(Self {
            id,
            tenant_id,
            datatable_id,
            name,
            description,
            columns,
            filter_criteria,
            sort_criteria,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DatatableError {
    #[error("registered table name must not be empty")]
    EmptyTableName,
    #[error("display name must not be empty")]
    EmptyDisplayName,
    #[error("column name must not be empty")]
    EmptyColumnName,
    #[error("column type must not be empty")]
    EmptyColumnType,
    #[error("name must not be empty")]
    EmptyName,
    #[error("columns must not be empty")]
    EmptyColumns,
    #[error("datatable not found: {0}")]
    DatatableNotFound(DatatableId),
    #[error("datatable column not found: {0}")]
    ColumnNotFound(DatatableColumnId),
    #[error("datatable view not found: {0}")]
    ViewNotFound(DatatableViewId),
    #[error("datatable {0} already exists")]
    AlreadyExists(DatatableId),
    #[error("tenant isolation violation: resource belongs to a different tenant")]
    TenantIsolationViolation,
    #[error("internal error: {detail}")]
    Internal { detail: String },
    #[error("query error: {detail}")]
    QueryError { detail: String },
}

impl DatatableError {
    pub fn is_tenant_isolation_violation(&self) -> bool {
        matches!(self, DatatableError::TenantIsolationViolation)
    }
}

impl From<sqlx::Error> for DatatableError {
    fn from(e: sqlx::Error) -> Self {
        DatatableError::QueryError {
            detail: e.to_string(),
        }
    }
}

impl From<fineract_security::SecurityError> for DatatableError {
    fn from(e: fineract_security::SecurityError) -> Self {
        DatatableError::Internal {
            detail: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;

    #[test]
    fn datatable_creation_validates() {
        let result = Datatable::new(
            DatatableId::new(1),
            TenantId::new(1),
            "".to_string(),
            "Display".to_string(),
            None,
            true,
        );
        assert!(matches!(result, Err(DatatableError::EmptyTableName)));

        let result = Datatable::new(
            DatatableId::new(1),
            TenantId::new(1),
            "table".to_string(),
            "".to_string(),
            None,
            true,
        );
        assert!(matches!(result, Err(DatatableError::EmptyDisplayName)));

        let dt = Datatable::new(
            DatatableId::new(1),
            TenantId::new(1),
            "my_table".to_string(),
            "My Table".to_string(),
            None,
            true,
        )
        .unwrap();
        assert_eq!(dt.registered_table_name, "my_table");
        assert_eq!(dt.display_name, "My Table");
    }

    #[test]
    fn datatable_column_creation_validates() {
        let result = DatatableColumn::new(
            DatatableColumnId::new(1),
            TenantId::new(1),
            DatatableId::new(1),
            "".to_string(),
            "Display".to_string(),
            "varchar".to_string(),
            None,
            None,
            None,
            false,
            false,
            false,
            None,
            None,
        );
        assert!(matches!(result, Err(DatatableError::EmptyColumnName)));

        let result = DatatableColumn::new(
            DatatableColumnId::new(1),
            TenantId::new(1),
            DatatableId::new(1),
            "col".to_string(),
            "".to_string(),
            "varchar".to_string(),
            None,
            None,
            None,
            false,
            false,
            false,
            None,
            None,
        );
        assert!(matches!(result, Err(DatatableError::EmptyDisplayName)));

        let result = DatatableColumn::new(
            DatatableColumnId::new(1),
            TenantId::new(1),
            DatatableId::new(1),
            "col".to_string(),
            "Display".to_string(),
            "".to_string(),
            None,
            None,
            None,
            false,
            false,
            false,
            None,
            None,
        );
        assert!(matches!(result, Err(DatatableError::EmptyColumnType)));
    }

    #[test]
    fn datatable_view_creation_validates() {
        let result = DatatableView::new(
            DatatableViewId::new(1),
            TenantId::new(1),
            DatatableId::new(1),
            "".to_string(),
            None,
            vec!["col1".to_string()],
            None,
            None,
        );
        assert!(matches!(result, Err(DatatableError::EmptyName)));

        let result = DatatableView::new(
            DatatableViewId::new(1),
            TenantId::new(1),
            DatatableId::new(1),
            "View".to_string(),
            None,
            vec![],
            None,
            None,
        );
        assert!(matches!(result, Err(DatatableError::EmptyColumns)));
    }
}
