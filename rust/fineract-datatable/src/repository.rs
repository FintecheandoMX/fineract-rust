//! Datatable repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{
    Datatable, DatatableColumn, DatatableColumnId, DatatableError, DatatableView, DatatableViewId,
};
use fineract_core::ids::DatatableId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type DatatableRepositoryResult<T> = Result<T, DatatableError>;

#[async_trait]
pub trait DatatableRepository: Send + Sync {
    // Datatable
    async fn find_datatable_by_id(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableRepositoryResult<Option<Datatable>>;
    async fn find_all_datatables(
        &self,
        context: &AuthorizedContext,
    ) -> DatatableRepositoryResult<Vec<Datatable>>;
    async fn insert_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: &Datatable,
    ) -> DatatableRepositoryResult<()>;
    async fn update_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: &Datatable,
    ) -> DatatableRepositoryResult<()>;
    async fn delete_datatable(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableRepositoryResult<()>;

    // DatatableColumn
    async fn find_columns_by_datatable_id(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableRepositoryResult<Vec<DatatableColumn>>;
    async fn find_column_by_id(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableRepositoryResult<Option<DatatableColumn>>;
    async fn insert_column(
        &self,
        context: &AuthorizedContext,
        column: &DatatableColumn,
    ) -> DatatableRepositoryResult<()>;
    async fn update_column(
        &self,
        context: &AuthorizedContext,
        column: &DatatableColumn,
    ) -> DatatableRepositoryResult<()>;
    async fn delete_column(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableRepositoryResult<()>;

    // DatatableView
    async fn find_views_by_datatable_id(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableRepositoryResult<Vec<DatatableView>>;
    async fn find_view_by_id(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableRepositoryResult<Option<DatatableView>>;
    async fn insert_view(
        &self,
        context: &AuthorizedContext,
        view: &DatatableView,
    ) -> DatatableRepositoryResult<()>;
    async fn update_view(
        &self,
        context: &AuthorizedContext,
        view: &DatatableView,
    ) -> DatatableRepositoryResult<()>;
    async fn delete_view(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableRepositoryResult<()>;
}

pub struct SqlxDatatableRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxDatatableRepository {
    pub fn new(
        pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
        connection_string: String,
    ) -> Self {
        Self {
            pool_manager,
            connection_string,
        }
    }

    async fn pool_for_context(
        &self,
        context: &AuthorizedContext,
    ) -> DatatableRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| DatatableError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl DatatableRepository for SqlxDatatableRepository {
    // --- Datatable ---

    async fn find_datatable_by_id(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableRepositoryResult<Option<Datatable>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, registered_table_name, display_name, description, multi_tenant, created_at, updated_at FROM datatables WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_datatable(&r)).transpose()
    }

    async fn find_all_datatables(
        &self,
        context: &AuthorizedContext,
    ) -> DatatableRepositoryResult<Vec<Datatable>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, registered_table_name, display_name, description, multi_tenant, created_at, updated_at FROM datatables WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_datatable(&r)).collect()
    }

    async fn insert_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: &Datatable,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO datatables (id, tenant_id, registered_table_name, display_name, description, multi_tenant, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(datatable.id.value())
        .bind(datatable.tenant_id.value())
        .bind(&datatable.registered_table_name)
        .bind(&datatable.display_name)
        .bind(&datatable.description)
        .bind(datatable.multi_tenant)
        .bind(datatable.created_at)
        .bind(datatable.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update_datatable(
        &self,
        context: &AuthorizedContext,
        datatable: &Datatable,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE datatables SET registered_table_name = $2, display_name = $3, description = $4, multi_tenant = $5, updated_at = $6 WHERE id = $1 AND tenant_id = $7",
        )
        .bind(datatable.id.value())
        .bind(&datatable.registered_table_name)
        .bind(&datatable.display_name)
        .bind(&datatable.description)
        .bind(datatable.multi_tenant)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(DatatableError::DatatableNotFound(datatable.id));
        }
        Ok(())
    }

    async fn delete_datatable(
        &self,
        context: &AuthorizedContext,
        id: DatatableId,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM datatables WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| DatatableError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(DatatableError::DatatableNotFound(id));
        }
        Ok(())
    }

    // --- DatatableColumn ---

    async fn find_columns_by_datatable_id(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableRepositoryResult<Vec<DatatableColumn>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, datatable_id, column_name, display_name, column_type, column_length, column_precision, column_scale, is_mandatory, is_unique, is_foreign_key, foreign_key_table, foreign_key_column, created_at, updated_at FROM datatable_columns WHERE datatable_id = $1 AND tenant_id = $2 ORDER BY id",
        )
        .bind(datatable_id.value())
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_column(&r)).collect()
    }

    async fn find_column_by_id(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableRepositoryResult<Option<DatatableColumn>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, datatable_id, column_name, display_name, column_type, column_length, column_precision, column_scale, is_mandatory, is_unique, is_foreign_key, foreign_key_table, foreign_key_column, created_at, updated_at FROM datatable_columns WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_column(&r)).transpose()
    }

    async fn insert_column(
        &self,
        context: &AuthorizedContext,
        column: &DatatableColumn,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO datatable_columns (id, tenant_id, datatable_id, column_name, display_name, column_type, column_length, column_precision, column_scale, is_mandatory, is_unique, is_foreign_key, foreign_key_table, foreign_key_column, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        )
        .bind(column.id.value())
        .bind(column.tenant_id.value())
        .bind(column.datatable_id.value())
        .bind(&column.column_name)
        .bind(&column.display_name)
        .bind(&column.column_type)
        .bind(column.column_length)
        .bind(column.column_precision)
        .bind(column.column_scale)
        .bind(column.is_mandatory)
        .bind(column.is_unique)
        .bind(column.is_foreign_key)
        .bind(&column.foreign_key_table)
        .bind(&column.foreign_key_column)
        .bind(column.created_at)
        .bind(column.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update_column(
        &self,
        context: &AuthorizedContext,
        column: &DatatableColumn,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE datatable_columns SET column_name = $2, display_name = $3, column_type = $4, column_length = $5, column_precision = $6, column_scale = $7, is_mandatory = $8, is_unique = $9, is_foreign_key = $10, foreign_key_table = $11, foreign_key_column = $12, updated_at = $13 WHERE id = $1 AND tenant_id = $14",
        )
        .bind(column.id.value())
        .bind(&column.column_name)
        .bind(&column.display_name)
        .bind(&column.column_type)
        .bind(column.column_length)
        .bind(column.column_precision)
        .bind(column.column_scale)
        .bind(column.is_mandatory)
        .bind(column.is_unique)
        .bind(column.is_foreign_key)
        .bind(&column.foreign_key_table)
        .bind(&column.foreign_key_column)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(DatatableError::ColumnNotFound(column.id));
        }
        Ok(())
    }

    async fn delete_column(
        &self,
        context: &AuthorizedContext,
        id: DatatableColumnId,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM datatable_columns WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| DatatableError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(DatatableError::ColumnNotFound(id));
        }
        Ok(())
    }

    // --- DatatableView ---

    async fn find_views_by_datatable_id(
        &self,
        context: &AuthorizedContext,
        datatable_id: DatatableId,
    ) -> DatatableRepositoryResult<Vec<DatatableView>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, datatable_id, name, description, columns, filter_criteria, sort_criteria, created_at, updated_at FROM datatable_views WHERE datatable_id = $1 AND tenant_id = $2 ORDER BY created_at DESC",
        )
        .bind(datatable_id.value())
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_view(&r)).collect()
    }

    async fn find_view_by_id(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableRepositoryResult<Option<DatatableView>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, datatable_id, name, description, columns, filter_criteria, sort_criteria, created_at, updated_at FROM datatable_views WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_view(&r)).transpose()
    }

    async fn insert_view(
        &self,
        context: &AuthorizedContext,
        view: &DatatableView,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO datatable_views (id, tenant_id, datatable_id, name, description, columns, filter_criteria, sort_criteria, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(view.id.value())
        .bind(view.tenant_id.value())
        .bind(view.datatable_id.value())
        .bind(&view.name)
        .bind(&view.description)
        .bind(&view.columns)
        .bind(&view.filter_criteria)
        .bind(&view.sort_criteria)
        .bind(view.created_at)
        .bind(view.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update_view(
        &self,
        context: &AuthorizedContext,
        view: &DatatableView,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE datatable_views SET name = $2, description = $3, columns = $4, filter_criteria = $5, sort_criteria = $6, updated_at = $7 WHERE id = $1 AND tenant_id = $8",
        )
        .bind(view.id.value())
        .bind(&view.name)
        .bind(&view.description)
        .bind(&view.columns)
        .bind(&view.filter_criteria)
        .bind(&view.sort_criteria)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| DatatableError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(DatatableError::ViewNotFound(view.id));
        }
        Ok(())
    }

    async fn delete_view(
        &self,
        context: &AuthorizedContext,
        id: DatatableViewId,
    ) -> DatatableRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM datatable_views WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| DatatableError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(DatatableError::ViewNotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_datatable(row: &sqlx::postgres::PgRow) -> DatatableRepositoryResult<Datatable> {
    Ok(Datatable {
        id: DatatableId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            DatatableError::QueryError {
                detail: "invalid datatable id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            DatatableError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        registered_table_name: row.try_get("registered_table_name")?,
        display_name: row.try_get("display_name")?,
        description: row.try_get("description")?,
        multi_tenant: row.try_get("multi_tenant")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn map_row_to_column(row: &sqlx::postgres::PgRow) -> DatatableRepositoryResult<DatatableColumn> {
    Ok(DatatableColumn {
        id: DatatableColumnId::new(row.try_get::<i64, _>("id")?),
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            DatatableError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        datatable_id: DatatableId::try_from(row.try_get::<i64, _>("datatable_id")?).map_err(
            |_| DatatableError::QueryError {
                detail: "invalid datatable id in database".to_string(),
            },
        )?,
        column_name: row.try_get("column_name")?,
        display_name: row.try_get("display_name")?,
        column_type: row.try_get("column_type")?,
        column_length: row.try_get("column_length")?,
        column_precision: row.try_get("column_precision")?,
        column_scale: row.try_get("column_scale")?,
        is_mandatory: row.try_get("is_mandatory")?,
        is_unique: row.try_get("is_unique")?,
        is_foreign_key: row.try_get("is_foreign_key")?,
        foreign_key_table: row.try_get("foreign_key_table")?,
        foreign_key_column: row.try_get("foreign_key_column")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn map_row_to_view(row: &sqlx::postgres::PgRow) -> DatatableRepositoryResult<DatatableView> {
    let columns: Vec<String> = row.try_get("columns")?;
    let filter_criteria: Option<serde_json::Value> = row.try_get("filter_criteria")?;
    let sort_criteria: Option<serde_json::Value> = row.try_get("sort_criteria")?;

    Ok(DatatableView {
        id: DatatableViewId::new(row.try_get::<i64, _>("id")?),
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            DatatableError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        datatable_id: DatatableId::try_from(row.try_get::<i64, _>("datatable_id")?).map_err(
            |_| DatatableError::QueryError {
                detail: "invalid datatable id in database".to_string(),
            },
        )?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        columns,
        filter_criteria,
        sort_criteria,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    #![allow(async_fn_in_trait)]
    use super::*;
    use fineract_core::ids::*;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryDatatableRepository {
        datatables: Mutex<HashMap<(TenantId, DatatableId), Datatable>>,
        columns: Mutex<HashMap<(TenantId, DatatableColumnId), DatatableColumn>>,
        views: Mutex<HashMap<(TenantId, DatatableViewId), DatatableView>>,
    }

    impl Default for InMemoryDatatableRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryDatatableRepository {
        pub fn new() -> Self {
            Self {
                datatables: Mutex::new(HashMap::new()),
                columns: Mutex::new(HashMap::new()),
                views: Mutex::new(HashMap::new()),
            }
        }

        fn dt_key(tenant_id: TenantId, id: DatatableId) -> (TenantId, DatatableId) {
            (tenant_id, id)
        }
        fn col_key(tenant_id: TenantId, id: DatatableColumnId) -> (TenantId, DatatableColumnId) {
            (tenant_id, id)
        }
        fn view_key(tenant_id: TenantId, id: DatatableViewId) -> (TenantId, DatatableViewId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl DatatableRepository for InMemoryDatatableRepository {
        async fn find_datatable_by_id(
            &self,
            context: &AuthorizedContext,
            id: DatatableId,
        ) -> DatatableRepositoryResult<Option<Datatable>> {
            let store = self.datatables.lock().unwrap();
            Ok(store.get(&Self::dt_key(context.tenant_id, id)).cloned())
        }

        async fn find_all_datatables(
            &self,
            context: &AuthorizedContext,
        ) -> DatatableRepositoryResult<Vec<Datatable>> {
            let store = self.datatables.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), d)| {
                    if *tid == context.tenant_id {
                        Some(d.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert_datatable(
            &self,
            context: &AuthorizedContext,
            datatable: &Datatable,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.datatables.lock().unwrap();
            store.insert(
                Self::dt_key(context.tenant_id, datatable.id),
                datatable.clone(),
            );
            Ok(())
        }

        async fn update_datatable(
            &self,
            context: &AuthorizedContext,
            datatable: &Datatable,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.datatables.lock().unwrap();
            if store.contains_key(&Self::dt_key(context.tenant_id, datatable.id)) {
                store.insert(
                    Self::dt_key(context.tenant_id, datatable.id),
                    datatable.clone(),
                );
                Ok(())
            } else {
                Err(DatatableError::DatatableNotFound(datatable.id))
            }
        }

        async fn delete_datatable(
            &self,
            context: &AuthorizedContext,
            id: DatatableId,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.datatables.lock().unwrap();
            if store.remove(&Self::dt_key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(DatatableError::DatatableNotFound(id))
            }
        }

        async fn find_columns_by_datatable_id(
            &self,
            context: &AuthorizedContext,
            datatable_id: DatatableId,
        ) -> DatatableRepositoryResult<Vec<DatatableColumn>> {
            let store = self.columns.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), c)| {
                    if *tid == context.tenant_id && c.datatable_id == datatable_id {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_column_by_id(
            &self,
            context: &AuthorizedContext,
            id: DatatableColumnId,
        ) -> DatatableRepositoryResult<Option<DatatableColumn>> {
            let store = self.columns.lock().unwrap();
            Ok(store.get(&Self::col_key(context.tenant_id, id)).cloned())
        }

        async fn insert_column(
            &self,
            context: &AuthorizedContext,
            column: &DatatableColumn,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.columns.lock().unwrap();
            store.insert(Self::col_key(context.tenant_id, column.id), column.clone());
            Ok(())
        }

        async fn update_column(
            &self,
            context: &AuthorizedContext,
            column: &DatatableColumn,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.columns.lock().unwrap();
            if store.contains_key(&Self::col_key(context.tenant_id, column.id)) {
                store.insert(Self::col_key(context.tenant_id, column.id), column.clone());
                Ok(())
            } else {
                Err(DatatableError::ColumnNotFound(column.id))
            }
        }

        async fn delete_column(
            &self,
            context: &AuthorizedContext,
            id: DatatableColumnId,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.columns.lock().unwrap();
            if store
                .remove(&Self::col_key(context.tenant_id, id))
                .is_some()
            {
                Ok(())
            } else {
                Err(DatatableError::ColumnNotFound(id))
            }
        }

        async fn find_views_by_datatable_id(
            &self,
            context: &AuthorizedContext,
            datatable_id: DatatableId,
        ) -> DatatableRepositoryResult<Vec<DatatableView>> {
            let store = self.views.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), v)| {
                    if *tid == context.tenant_id && v.datatable_id == datatable_id {
                        Some(v.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_view_by_id(
            &self,
            context: &AuthorizedContext,
            id: DatatableViewId,
        ) -> DatatableRepositoryResult<Option<DatatableView>> {
            let store = self.views.lock().unwrap();
            Ok(store.get(&Self::view_key(context.tenant_id, id)).cloned())
        }

        async fn insert_view(
            &self,
            context: &AuthorizedContext,
            view: &DatatableView,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.views.lock().unwrap();
            store.insert(Self::view_key(context.tenant_id, view.id), view.clone());
            Ok(())
        }

        async fn update_view(
            &self,
            context: &AuthorizedContext,
            view: &DatatableView,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.views.lock().unwrap();
            if store.contains_key(&Self::view_key(context.tenant_id, view.id)) {
                store.insert(Self::view_key(context.tenant_id, view.id), view.clone());
                Ok(())
            } else {
                Err(DatatableError::ViewNotFound(view.id))
            }
        }

        async fn delete_view(
            &self,
            context: &AuthorizedContext,
            id: DatatableViewId,
        ) -> DatatableRepositoryResult<()> {
            let mut store = self.views.lock().unwrap();
            if store
                .remove(&Self::view_key(context.tenant_id, id))
                .is_some()
            {
                Ok(())
            } else {
                Err(DatatableError::ViewNotFound(id))
            }
        }
    }

    fn make_context(tenant_id: i64) -> AuthorizedContext {
        AuthorizedContext::new(
            TenantId::new(tenant_id),
            fineract_security::Principal {
                app_user_id: AppUserId::new(1),
                username: "tester".into(),
                roles: HashSet::new(),
                permissions: HashSet::new(),
                office_id: None,
            },
            SessionId::new(),
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn datatable_repository_is_tenant_isolated() {
        let repo = InMemoryDatatableRepository::new();
        let dt = Datatable::new(
            DatatableId::new(42),
            TenantId::new(1),
            "test_table".to_string(),
            "Test Table".to_string(),
            None,
            true,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert_datatable(&ctx, &dt).await.unwrap();
        let result = repo
            .find_datatable_by_id(&ctx, DatatableId::new(42))
            .await
            .unwrap();
        assert_eq!(result.unwrap().display_name, "Test Table");

        let ctx2 = make_context(2);
        let result = repo
            .find_datatable_by_id(&ctx2, DatatableId::new(42))
            .await
            .unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn datatable_column_repository_tenant_isolated() {
        let repo = InMemoryDatatableRepository::new();
        let col = DatatableColumn::new(
            DatatableColumnId::new(1),
            TenantId::new(1),
            DatatableId::new(1),
            "col1".to_string(),
            "Column 1".to_string(),
            "varchar".to_string(),
            Some(100),
            None,
            None,
            false,
            false,
            false,
            None,
            None,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert_column(&ctx, &col).await.unwrap();
        let result = repo
            .find_column_by_id(&ctx, DatatableColumnId::new(1))
            .await
            .unwrap();
        assert_eq!(result.unwrap().column_name, "col1");

        let ctx2 = make_context(2);
        let result = repo
            .find_column_by_id(&ctx2, DatatableColumnId::new(1))
            .await
            .unwrap();
        assert_eq!(result, None);
    }
}
