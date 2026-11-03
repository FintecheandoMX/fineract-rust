//! Note repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Note, NoteEntityType, NoteError};
use fineract_core::ids::NoteId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type NoteRepositoryResult<T> = Result<T, NoteError>;

#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: NoteId,
    ) -> NoteRepositoryResult<Option<Note>>;
    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> NoteRepositoryResult<Vec<Note>>;
    async fn find_by_entity(
        &self,
        context: &AuthorizedContext,
        entity_type: NoteEntityType,
        entity_id: i64,
    ) -> NoteRepositoryResult<Vec<Note>>;
    async fn insert(&self, context: &AuthorizedContext, note: &Note) -> NoteRepositoryResult<()>;
    async fn update(&self, context: &AuthorizedContext, note: &Note) -> NoteRepositoryResult<()>;
    async fn delete(&self, context: &AuthorizedContext, id: NoteId) -> NoteRepositoryResult<()>;
}

pub struct SqlxNoteRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxNoteRepository {
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
    ) -> NoteRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| NoteError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl NoteRepository for SqlxNoteRepository {
    async fn find_by_id(
        &self,
        context: &AuthorizedContext,
        id: NoteId,
    ) -> NoteRepositoryResult<Option<Note>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, entity_type, entity_id, note, created_by, created_at, updated_at, active FROM notes WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| NoteError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_note(&r)).transpose()
    }

    async fn find_all_by_tenant(
        &self,
        context: &AuthorizedContext,
    ) -> NoteRepositoryResult<Vec<Note>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, entity_type, entity_id, note, created_by, created_at, updated_at, active FROM notes WHERE tenant_id = $1 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| NoteError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_note(&r)).collect()
    }

    async fn find_by_entity(
        &self,
        context: &AuthorizedContext,
        entity_type: NoteEntityType,
        entity_id: i64,
    ) -> NoteRepositoryResult<Vec<Note>> {
        let pool = self.pool_for_context(context).await?;
        let entity_type_str = match entity_type {
            NoteEntityType::Client => "client",
            NoteEntityType::Loan => "loan",
            NoteEntityType::Savings => "savings",
            NoteEntityType::Group => "group",
            NoteEntityType::Center => "center",
        };
        let rows = sqlx::query(
            "SELECT id, tenant_id, entity_type, entity_id, note, created_by, created_at, updated_at, active FROM notes WHERE tenant_id = $1 AND entity_type = $2 AND entity_id = $3 ORDER BY created_at DESC",
        )
        .bind(context.tenant_id.value())
        .bind(entity_type_str)
        .bind(entity_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| NoteError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_note(&r)).collect()
    }

    async fn insert(&self, context: &AuthorizedContext, note: &Note) -> NoteRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let entity_type_str = match note.entity_type {
            NoteEntityType::Client => "client",
            NoteEntityType::Loan => "loan",
            NoteEntityType::Savings => "savings",
            NoteEntityType::Group => "group",
            NoteEntityType::Center => "center",
        };
        sqlx::query(
            "INSERT INTO notes (id, tenant_id, entity_type, entity_id, note, created_by, created_at, updated_at, active) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(note.id.value())
        .bind(note.tenant_id.value())
        .bind(entity_type_str)
        .bind(note.entity_id)
        .bind(&note.note)
        .bind(note.created_by)
        .bind(note.created_at)
        .bind(note.updated_at)
        .bind(note.active)
        .execute(&pool)
        .await
        .map_err(|e| NoteError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update(&self, context: &AuthorizedContext, note: &Note) -> NoteRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let entity_type_str = match note.entity_type {
            NoteEntityType::Client => "client",
            NoteEntityType::Loan => "loan",
            NoteEntityType::Savings => "savings",
            NoteEntityType::Group => "group",
            NoteEntityType::Center => "center",
        };
        let result = sqlx::query(
            "UPDATE notes SET entity_type = $2, entity_id = $3, note = $4, created_by = $5, updated_at = $6, active = $7 WHERE id = $1 AND tenant_id = $8",
        )
        .bind(note.id.value())
        .bind(entity_type_str)
        .bind(note.entity_id)
        .bind(&note.note)
        .bind(note.created_by)
        .bind(chrono::Utc::now())
        .bind(note.active)
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| NoteError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(NoteError::NotFound(note.id));
        }
        Ok(())
    }

    async fn delete(&self, context: &AuthorizedContext, id: NoteId) -> NoteRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| NoteError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(NoteError::NotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_note(row: &sqlx::postgres::PgRow) -> NoteRepositoryResult<Note> {
    let entity_type_str: String = row.try_get("entity_type")?;
    let entity_type = match entity_type_str.as_str() {
        "client" => NoteEntityType::Client,
        "loan" => NoteEntityType::Loan,
        "savings" => NoteEntityType::Savings,
        "group" => NoteEntityType::Group,
        "center" => NoteEntityType::Center,
        _ => {
            return Err(NoteError::QueryError {
                detail: format!("invalid entity_type in database: {entity_type_str}"),
            });
        }
    };

    Ok(Note {
        id: NoteId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| NoteError::QueryError {
            detail: "invalid note id in database".to_string(),
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            NoteError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        entity_type,
        entity_id: row.try_get("entity_id")?,
        note: row.try_get("note")?,
        created_by: row.try_get("created_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        active: row.try_get("active")?,
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

    pub struct InMemoryNoteRepository {
        store: Mutex<HashMap<(TenantId, NoteId), Note>>,
    }

    impl Default for InMemoryNoteRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryNoteRepository {
        pub fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }

        fn key(tenant_id: TenantId, id: NoteId) -> (TenantId, NoteId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl NoteRepository for InMemoryNoteRepository {
        async fn find_by_id(
            &self,
            context: &AuthorizedContext,
            id: NoteId,
        ) -> NoteRepositoryResult<Option<Note>> {
            let store = self.store.lock().unwrap();
            Ok(store.get(&Self::key(context.tenant_id, id)).cloned())
        }

        async fn find_all_by_tenant(
            &self,
            context: &AuthorizedContext,
        ) -> NoteRepositoryResult<Vec<Note>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), n)| {
                    if *tid == context.tenant_id {
                        Some(n.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_by_entity(
            &self,
            context: &AuthorizedContext,
            entity_type: NoteEntityType,
            entity_id: i64,
        ) -> NoteRepositoryResult<Vec<Note>> {
            let store = self.store.lock().unwrap();
            Ok(store
                .iter()
                .filter_map(|((tid, _), n)| {
                    if *tid == context.tenant_id
                        && n.entity_type == entity_type
                        && n.entity_id == entity_id
                    {
                        Some(n.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn insert(
            &self,
            context: &AuthorizedContext,
            note: &Note,
        ) -> NoteRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(Self::key(context.tenant_id, note.id), note.clone());
            Ok(())
        }

        async fn update(
            &self,
            context: &AuthorizedContext,
            note: &Note,
        ) -> NoteRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.contains_key(&Self::key(context.tenant_id, note.id)) {
                store.insert(Self::key(context.tenant_id, note.id), note.clone());
                Ok(())
            } else {
                Err(NoteError::NotFound(note.id))
            }
        }

        async fn delete(
            &self,
            context: &AuthorizedContext,
            id: NoteId,
        ) -> NoteRepositoryResult<()> {
            let mut store = self.store.lock().unwrap();
            if store.remove(&Self::key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(NoteError::NotFound(id))
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
    async fn note_repository_is_tenant_isolated() {
        let repo = InMemoryNoteRepository::new();
        let note = Note::new(
            NoteId::new(42),
            TenantId::new(1),
            NoteEntityType::Client,
            100,
            "Test note".to_string(),
            1,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &note).await.unwrap();
        let result = repo.find_by_id(&ctx, NoteId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().note, "Test note");

        let ctx2 = make_context(2);
        let result = repo.find_by_id(&ctx2, NoteId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn note_repository_find_by_entity() {
        let repo = InMemoryNoteRepository::new();
        let note1 = Note::new(
            NoteId::new(1),
            TenantId::new(1),
            NoteEntityType::Client,
            100,
            "Client note".to_string(),
            1,
        )
        .unwrap();
        let note2 = Note::new(
            NoteId::new(2),
            TenantId::new(1),
            NoteEntityType::Loan,
            200,
            "Loan note".to_string(),
            1,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert(&ctx, &note1).await.unwrap();
        repo.insert(&ctx, &note2).await.unwrap();

        let client_notes = repo
            .find_by_entity(&ctx, NoteEntityType::Client, 100)
            .await
            .unwrap();
        assert_eq!(client_notes.len(), 1);
        assert_eq!(client_notes[0].note, "Client note");

        let loan_notes = repo
            .find_by_entity(&ctx, NoteEntityType::Loan, 200)
            .await
            .unwrap();
        assert_eq!(loan_notes.len(), 1);
        assert_eq!(loan_notes[0].note, "Loan note");
    }
}
