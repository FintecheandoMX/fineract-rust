//! Document domain service.

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Document, DocumentError};
use crate::repository::DocumentRepository;
use fineract_core::ids::DocumentId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type DocumentServiceResult<T> = Result<T, DocumentError>;

#[async_trait]
pub trait DocumentService: Send + Sync {
    async fn create_document(
        &self,
        context: &AuthorizedContext,
        doc: Document,
    ) -> DocumentServiceResult<Document>;
    async fn get_document(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentServiceResult<Option<Document>>;
    async fn list_documents(
        &self,
        context: &AuthorizedContext,
    ) -> DocumentServiceResult<Vec<Document>>;
    async fn update_document(
        &self,
        context: &AuthorizedContext,
        doc: Document,
    ) -> DocumentServiceResult<Document>;
    async fn delete_document(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentServiceResult<DocumentId>;
}

pub struct DocumentServiceImpl<R: DocumentRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: DocumentRepository> DocumentServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: DocumentRepository> DocumentService for DocumentServiceImpl<R> {
    async fn create_document(
        &self,
        context: &AuthorizedContext,
        doc: Document,
    ) -> DocumentServiceResult<Document> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DocumentError::Internal {
                detail: e.to_string(),
            })?;
        if doc.tenant_id != context.tenant_id {
            return Err(DocumentError::TenantIsolationViolation);
        }
        self.repository.insert(context, &doc).await?;
        Ok(doc)
    }

    async fn get_document(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentServiceResult<Option<Document>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DocumentError::Internal {
                detail: e.to_string(),
            })?;
        let doc = self.repository.find_by_id(context, id).await?;
        if let Some(ref d) = doc {
            if d.tenant_id != context.tenant_id {
                return Err(DocumentError::TenantIsolationViolation);
            }
        }
        Ok(doc)
    }

    async fn list_documents(
        &self,
        context: &AuthorizedContext,
    ) -> DocumentServiceResult<Vec<Document>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| DocumentError::Internal {
                detail: e.to_string(),
            })?;
        self.repository.find_all_by_tenant(context).await
    }

    async fn update_document(
        &self,
        context: &AuthorizedContext,
        doc: Document,
    ) -> DocumentServiceResult<Document> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DocumentError::Internal {
                detail: e.to_string(),
            })?;
        if doc.tenant_id != context.tenant_id {
            return Err(DocumentError::TenantIsolationViolation);
        }
        self.repository.update(context, &doc).await?;
        Ok(doc)
    }

    async fn delete_document(
        &self,
        context: &AuthorizedContext,
        id: DocumentId,
    ) -> DocumentServiceResult<DocumentId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| DocumentError::Internal {
                detail: e.to_string(),
            })?;
        let doc = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(DocumentError::NotFound(id))?;
        if doc.tenant_id != context.tenant_id {
            return Err(DocumentError::TenantIsolationViolation);
        }
        self.repository.delete(context, id).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn DocumentService) {}
    }
}
