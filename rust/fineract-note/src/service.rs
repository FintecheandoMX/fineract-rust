//! Note domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Note, NoteEntityType, NoteError};
use crate::repository::NoteRepository;
use fineract_core::ids::NoteId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type NoteServiceResult<T> = Result<T, NoteError>;

#[async_trait]
pub trait NoteService: Send + Sync {
    async fn create_note(&self, context: &AuthorizedContext, note: Note)
        -> NoteServiceResult<Note>;
    async fn get_note(
        &self,
        context: &AuthorizedContext,
        id: NoteId,
    ) -> NoteServiceResult<Option<Note>>;
    async fn list_notes(&self, context: &AuthorizedContext) -> NoteServiceResult<Vec<Note>>;
    async fn list_notes_by_entity(
        &self,
        context: &AuthorizedContext,
        entity_type: NoteEntityType,
        entity_id: i64,
    ) -> NoteServiceResult<Vec<Note>>;
    async fn update_note(&self, context: &AuthorizedContext, note: Note)
        -> NoteServiceResult<Note>;
    async fn delete_note(
        &self,
        context: &AuthorizedContext,
        id: NoteId,
    ) -> NoteServiceResult<NoteId>;
}

pub struct NoteServiceImpl<R: NoteRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: NoteRepository> NoteServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: NoteRepository> NoteService for NoteServiceImpl<R> {
    async fn create_note(
        &self,
        context: &AuthorizedContext,
        note: Note,
    ) -> NoteServiceResult<Note> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| NoteError::Internal {
                detail: e.to_string(),
            })?;

        if note.tenant_id != context.tenant_id {
            return Err(NoteError::TenantIsolationViolation);
        }

        self.repository.insert(context, &note).await?;
        Ok(note)
    }

    async fn get_note(
        &self,
        context: &AuthorizedContext,
        id: NoteId,
    ) -> NoteServiceResult<Option<Note>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| NoteError::Internal {
                detail: e.to_string(),
            })?;

        let note = self.repository.find_by_id(context, id).await?;

        if let Some(ref n) = note {
            if n.tenant_id != context.tenant_id {
                return Err(NoteError::TenantIsolationViolation);
            }
        }

        Ok(note)
    }

    async fn list_notes(&self, context: &AuthorizedContext) -> NoteServiceResult<Vec<Note>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| NoteError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_by_tenant(context).await
    }

    async fn list_notes_by_entity(
        &self,
        context: &AuthorizedContext,
        entity_type: NoteEntityType,
        entity_id: i64,
    ) -> NoteServiceResult<Vec<Note>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| NoteError::Internal {
                detail: e.to_string(),
            })?;

        self.repository
            .find_by_entity(context, entity_type, entity_id)
            .await
    }

    async fn update_note(
        &self,
        context: &AuthorizedContext,
        note: Note,
    ) -> NoteServiceResult<Note> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| NoteError::Internal {
                detail: e.to_string(),
            })?;

        if note.tenant_id != context.tenant_id {
            return Err(NoteError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_by_id(context, note.id)
            .await?
            .ok_or(NoteError::NotFound(note.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(NoteError::TenantIsolationViolation);
        }

        self.repository.update(context, &note).await?;
        Ok(note)
    }

    async fn delete_note(
        &self,
        context: &AuthorizedContext,
        id: NoteId,
    ) -> NoteServiceResult<NoteId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| NoteError::Internal {
                detail: e.to_string(),
            })?;

        let note = self
            .repository
            .find_by_id(context, id)
            .await?
            .ok_or(NoteError::NotFound(id))?;

        if note.tenant_id != context.tenant_id {
            return Err(NoteError::TenantIsolationViolation);
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
        fn _assert_object_safe(_: &dyn NoteService) {}
    }
}
