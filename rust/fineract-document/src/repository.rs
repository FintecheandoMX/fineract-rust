//! Document repository.

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Document, DocumentError};
use fineract_core::ids::DocumentId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type DocumentRepositoryResult<T> = Result<T, DocumentError>;

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentRepositoryResult<Option<Document>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> DocumentRepositoryResult<Vec<Document>>;
    async fn insert(
        &self,
        context: &AuthorizedContext,
        doc: &Document,
    ) -> DocumentRepositoryResult<()>;
    async fn update(
        &self,
        context: &AuthorizedContext,
        doc: &Document,
    ) -> DocumentRepositoryResult<()>;
    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentRepositoryResult<()>;
}

pub struct SqlxDocumentRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxDocumentRepository {
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
    ) -> DocumentRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| DocumentError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl DocumentRepository for SqlxDocumentRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentRepositoryResult<Option<Document>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, content_type, storage_path, size_bytes, active FROM documents WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| DocumentError::QueryError { detail: e.to_string() })?;
        row.map(|r| map_row_to_document(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> DocumentRepositoryResult<Vec<Document>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, content_type, storage_path, size_bytes, active FROM documents WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| DocumentError::QueryError { detail: e.to_string() })?;
        rows.into_iter().map(|r| map_row_to_document(&r)).collect()
    }

    async fn insert(
        &self,
        context: &AuthorizedContext,
        doc: &Document,
    ) -> DocumentRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO documents (id, tenant_id, name, content_type, storage_path, size_bytes, active) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(doc.id.value())
        .bind(doc.tenant_id.value())
        .bind(&doc.name)
        .bind(&doc.content_type)
        .bind(&doc.storage_path)
        .bind(doc.size_bytes)
        .bind(doc.active)
        .execute(&pool)
        .await
        .map_err(|e| DocumentError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(
        &self,
        context: &AuthorizedContext,
        doc: &Document,
    ) -> DocumentRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE documents SET name = $2, content_type = $3, storage_path = $4, size_bytes = $5, active = $6 WHERE id = $1 AND tenant_id = $7",
        )
        .bind(doc.id.value())
        .bind(&doc.name)
        .bind(&doc.content_type)
        .bind(&doc.storage_path)
        .bind(doc.size_bytes)
        .bind(doc.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| DocumentError::QueryError { detail: e.to_string() })?;
        if result.rows_affected() == 0 {
            return Err(DocumentError::NotFound(doc.id));
        }
        Ok(())
    }

    async fn delete(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM documents WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| DocumentError::QueryError {
                detail: e.to_string(),
            })?;
        if result.rows_affected() == 0 {
            return Err(DocumentError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_document(row: &sqlx::postgres::PgRow) -> DocumentRepositoryResult<Document> {
    Ok(Document {
        id: DocumentId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            DocumentError::QueryError {
                detail: "invalid document id".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            DocumentError::QueryError {
                detail: "invalid tenant id".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        content_type: row.try_get("content_type")?,
        storage_path: row.try_get("storage_path")?,
        size_bytes: row.try_get("size_bytes")?,
        active: row.try_get("active")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fineract_core::ids::*;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryDocumentRepository {
        store: Mutex<HashMap<(TenantId, DocumentId), Document>>,
    }

    impl Default for InMemoryDocumentRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryDocumentRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: DocumentId) -> (TenantId, DocumentId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl DocumentRepository for InMemoryDocumentRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: DocumentId,
        ) -> DocumentRepositoryResult<Option<Document>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> DocumentRepositoryResult<Vec<Document>> {
            let store = self.store.lock().unwrap();
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

        async fn insert(
            &self,
            context: &AuthorizedContext,
            doc: &Document,
        ) -> DocumentRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, doc.id), doc.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            doc: &Document,
        ) -> DocumentRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, doc.id)) {
                store.insert(Self::key(context.tenant_id, doc.id), doc.clone());
                Ok(())
            } else {
                Err(DocumentError::NotFound(doc.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: DocumentId,
        ) -> DocumentRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(DocumentError::NotFound(id))
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
    async fn document_repository_is_tenant_isolated() {
        let repo = InMemoryDocumentRepository::new();
        let doc = Document::new(
            DocumentId::new(42),
            TenantId::new(1),
            "Contract".to_string(),
            "application/pdf".to_string(),
            "/docs/1.pdf".to_string(),
            1024,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &doc).await.unwrap();
        let result = repo.find_by_id(&ctx, DocumentId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Contract");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, DocumentId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }
}
