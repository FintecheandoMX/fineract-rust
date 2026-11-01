//! Code and CodeValue repository.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use sqlx::Row;

use crate::domain::{Code, CodeError, CodeValue};
use fineract_core::ids::CodeId;
use fineract_core::ids::CodeValueId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

pub type CodeRepositoryResult<T> = Result<T, CodeError>;

#[async_trait]
pub trait CodeRepository: Send + Sync {
    async fn find_code_by_id(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeRepositoryResult<Option<Code>>;
    async fn find_all_codes(&self, context: &AuthorizedContext) -> CodeRepositoryResult<Vec<Code>>;
    async fn find_code_values_by_code_id(
        &self,
        context: &AuthorizedContext,
        code_id: CodeId,
    ) -> CodeRepositoryResult<Vec<CodeValue>>;
    async fn find_code_value_by_id(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeRepositoryResult<Option<CodeValue>>;
    async fn insert_code(
        &self,
        context: &AuthorizedContext,
        code: &Code,
    ) -> CodeRepositoryResult<()>;
    async fn update_code(
        &self,
        context: &AuthorizedContext,
        code: &Code,
    ) -> CodeRepositoryResult<()>;
    async fn delete_code(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeRepositoryResult<()>;
    async fn insert_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: &CodeValue,
    ) -> CodeRepositoryResult<()>;
    async fn update_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: &CodeValue,
    ) -> CodeRepositoryResult<()>;
    async fn delete_code_value(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeRepositoryResult<()>;
}

pub struct SqlxCodeRepository {
    pool_manager: std::sync::Arc<fineract_persistence::TenantPoolManager>,
    connection_string: String,
}

impl SqlxCodeRepository {
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
    ) -> CodeRepositoryResult<sqlx::Pool<sqlx::Postgres>> {
        self.pool_manager
            .pool_for_tenant(context.tenant_id, self.connection_string.clone())
            .await
            .map_err(|e| CodeError::Internal {
                detail: format!(
                    "failed to get pool for tenant {}: {e}",
                    context.tenant_id.value()
                ),
            })
    }
}

#[async_trait]
impl CodeRepository for SqlxCodeRepository {
    async fn find_code_by_id(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeRepositoryResult<Option<Code>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, tenant_id, name, description, is_active, created_at, updated_at FROM codes WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_code(&r)).transpose()
    }

    async fn find_all_codes(&self, context: &AuthorizedContext) -> CodeRepositoryResult<Vec<Code>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, tenant_id, name, description, is_active, created_at, updated_at FROM codes WHERE tenant_id = $1 ORDER BY name",
        )
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;

        rows.into_iter().map(|r| map_row_to_code(&r)).collect()
    }

    async fn find_code_values_by_code_id(
        &self,
        context: &AuthorizedContext,
        code_id: CodeId,
    ) -> CodeRepositoryResult<Vec<CodeValue>> {
        let pool = self.pool_for_context(context).await?;
        let rows = sqlx::query(
            "SELECT id, code_id, tenant_id, name, description, position, is_active, created_at, updated_at FROM code_values WHERE code_id = $1 AND tenant_id = $2 ORDER BY position",
        )
        .bind(code_id.value())
        .bind(context.tenant_id.value())
        .fetch_all(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;

        rows.into_iter()
            .map(|r| map_row_to_code_value(&r))
            .collect()
    }

    async fn find_code_value_by_id(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeRepositoryResult<Option<CodeValue>> {
        let pool = self.pool_for_context(context).await?;
        let row = sqlx::query(
            "SELECT id, code_id, tenant_id, name, description, position, is_active, created_at, updated_at FROM code_values WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.value())
        .bind(context.tenant_id.value())
        .fetch_optional(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;

        row.map(|r| map_row_to_code_value(&r)).transpose()
    }

    async fn insert_code(
        &self,
        context: &AuthorizedContext,
        code: &Code,
    ) -> CodeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO codes (id, tenant_id, name, description, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(code.id.value())
        .bind(code.tenant_id.value())
        .bind(&code.name)
        .bind(&code.description)
        .bind(code.is_active)
        .bind(code.created_at)
        .bind(code.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update_code(
        &self,
        context: &AuthorizedContext,
        code: &Code,
    ) -> CodeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE codes SET name = $2, description = $3, is_active = $4, updated_at = $5 WHERE id = $1 AND tenant_id = $6",
        )
        .bind(code.id.value())
        .bind(&code.name)
        .bind(&code.description)
        .bind(code.is_active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(CodeError::NotFound(code.id));
        }
        Ok(())
    }

    async fn delete_code(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM codes WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| CodeError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(CodeError::NotFound(id));
        }
        Ok(())
    }

    async fn insert_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: &CodeValue,
    ) -> CodeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        sqlx::query(
            "INSERT INTO code_values (id, code_id, tenant_id, name, description, position, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(code_value.id.value())
        .bind(code_value.code_id.value())
        .bind(code_value.tenant_id.value())
        .bind(&code_value.name)
        .bind(&code_value.description)
        .bind(code_value.position)
        .bind(code_value.is_active)
        .bind(code_value.created_at)
        .bind(code_value.updated_at)
        .execute(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;
        Ok(())
    }

    async fn update_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: &CodeValue,
    ) -> CodeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query(
            "UPDATE code_values SET name = $2, description = $3, position = $4, is_active = $5, updated_at = $6 WHERE id = $1 AND tenant_id = $7",
        )
        .bind(code_value.id.value())
        .bind(&code_value.name)
        .bind(&code_value.description)
        .bind(code_value.position)
        .bind(code_value.is_active)
        .bind(chrono::Utc::now())
        .bind(context.tenant_id.value())
        .execute(&pool)
        .await
        .map_err(|e| CodeError::QueryError { detail: e.to_string() })?;

        if result.rows_affected() == 0 {
            return Err(CodeError::CodeValueNotFound(code_value.id));
        }
        Ok(())
    }

    async fn delete_code_value(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeRepositoryResult<()> {
        let pool = self.pool_for_context(context).await?;
        let result = sqlx::query("DELETE FROM code_values WHERE id = $1 AND tenant_id = $2")
            .bind(id.value())
            .bind(context.tenant_id.value())
            .execute(&pool)
            .await
            .map_err(|e| CodeError::QueryError {
                detail: e.to_string(),
            })?;

        if result.rows_affected() == 0 {
            return Err(CodeError::CodeValueNotFound(id));
        }
        Ok(())
    }
}

fn map_row_to_code(row: &sqlx::postgres::PgRow) -> CodeRepositoryResult<Code> {
    Ok(Code {
        id: CodeId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| CodeError::QueryError {
            detail: "invalid code id in database".to_string(),
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            CodeError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn map_row_to_code_value(row: &sqlx::postgres::PgRow) -> CodeRepositoryResult<CodeValue> {
    Ok(CodeValue {
        id: CodeValueId::try_from(row.try_get::<i64, _>("id")?).map_err(|_| {
            CodeError::QueryError {
                detail: "invalid code value id in database".to_string(),
            }
        })?,
        code_id: CodeId::try_from(row.try_get::<i64, _>("code_id")?).map_err(|_| {
            CodeError::QueryError {
                detail: "invalid code id in database".to_string(),
            }
        })?,
        tenant_id: TenantId::try_from(row.try_get::<i64, _>("tenant_id")?).map_err(|_| {
            CodeError::QueryError {
                detail: "invalid tenant id in database".to_string(),
            }
        })?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        position: row.try_get("position")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    #![allow(async_fn_in_trait)]
    use super::*;
    use chrono::Datelike;
    use fineract_core::ids::*;
    use fineract_security::SessionId;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Mutex;

    pub struct InMemoryCodeRepository {
        codes: Mutex<HashMap<(TenantId, CodeId), Code>>,
        code_values: Mutex<HashMap<(TenantId, CodeValueId), CodeValue>>,
    }

    impl Default for InMemoryCodeRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InMemoryCodeRepository {
        pub fn new() -> Self {
            Self {
                codes: Mutex::new(HashMap::new()),
                code_values: Mutex::new(HashMap::new()),
            }
        }

        fn code_key(tenant_id: TenantId, id: CodeId) -> (TenantId, CodeId) {
            (tenant_id, id)
        }

        fn cv_key(tenant_id: TenantId, id: CodeValueId) -> (TenantId, CodeValueId) {
            (tenant_id, id)
        }
    }

    #[async_trait]
    impl CodeRepository for InMemoryCodeRepository {
        async fn find_code_by_id(
            &self,
            context: &AuthorizedContext,
            id: CodeId,
        ) -> CodeRepositoryResult<Option<Code>> {
            let codes = self.codes.lock().unwrap();
            Ok(codes.get(&Self::code_key(context.tenant_id, id)).cloned())
        }

        async fn find_all_codes(
            &self,
            context: &AuthorizedContext,
        ) -> CodeRepositoryResult<Vec<Code>> {
            let codes = self.codes.lock().unwrap();
            Ok(codes
                .iter()
                .filter_map(|((tid, _), c)| {
                    if *tid == context.tenant_id {
                        Some(c.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_code_values_by_code_id(
            &self,
            context: &AuthorizedContext,
            code_id: CodeId,
        ) -> CodeRepositoryResult<Vec<CodeValue>> {
            let cvs = self.code_values.lock().unwrap();
            Ok(cvs
                .iter()
                .filter_map(|((tid, _), cv)| {
                    if *tid == context.tenant_id && cv.code_id == code_id {
                        Some(cv.clone())
                    } else {
                        None
                    }
                })
                .collect())
        }

        async fn find_code_value_by_id(
            &self,
            context: &AuthorizedContext,
            id: CodeValueId,
        ) -> CodeRepositoryResult<Option<CodeValue>> {
            let cvs = self.code_values.lock().unwrap();
            Ok(cvs.get(&Self::cv_key(context.tenant_id, id)).cloned())
        }

        async fn insert_code(
            &self,
            context: &AuthorizedContext,
            code: &Code,
        ) -> CodeRepositoryResult<()> {
            let mut codes = self.codes.lock().unwrap();
            codes.insert(Self::code_key(context.tenant_id, code.id), code.clone());
            Ok(())
        }

        async fn update_code(
            &self,
            context: &AuthorizedContext,
            code: &Code,
        ) -> CodeRepositoryResult<()> {
            let mut codes = self.codes.lock().unwrap();
            if codes.contains_key(&Self::code_key(context.tenant_id, code.id)) {
                codes.insert(Self::code_key(context.tenant_id, code.id), code.clone());
                Ok(())
            } else {
                Err(CodeError::NotFound(code.id))
            }
        }

        async fn delete_code(
            &self,
            context: &AuthorizedContext,
            id: CodeId,
        ) -> CodeRepositoryResult<()> {
            let mut codes = self.codes.lock().unwrap();
            if codes
                .remove(&Self::code_key(context.tenant_id, id))
                .is_some()
            {
                Ok(())
            } else {
                Err(CodeError::NotFound(id))
            }
        }

        async fn insert_code_value(
            &self,
            context: &AuthorizedContext,
            code_value: &CodeValue,
        ) -> CodeRepositoryResult<()> {
            let mut cvs = self.code_values.lock().unwrap();
            cvs.insert(
                Self::cv_key(context.tenant_id, code_value.id),
                code_value.clone(),
            );
            Ok(())
        }

        async fn update_code_value(
            &self,
            context: &AuthorizedContext,
            code_value: &CodeValue,
        ) -> CodeRepositoryResult<()> {
            let mut cvs = self.code_values.lock().unwrap();
            if cvs.contains_key(&Self::cv_key(context.tenant_id, code_value.id)) {
                cvs.insert(
                    Self::cv_key(context.tenant_id, code_value.id),
                    code_value.clone(),
                );
                Ok(())
            } else {
                Err(CodeError::CodeValueNotFound(code_value.id))
            }
        }

        async fn delete_code_value(
            &self,
            context: &AuthorizedContext,
            id: CodeValueId,
        ) -> CodeRepositoryResult<()> {
            let mut cvs = self.code_values.lock().unwrap();
            if cvs.remove(&Self::cv_key(context.tenant_id, id)).is_some() {
                Ok(())
            } else {
                Err(CodeError::CodeValueNotFound(id))
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
    async fn code_repository_is_tenant_isolated() {
        let repo = InMemoryCodeRepository::new();
        let code = Code::new(
            CodeId::new(42),
            TenantId::new(1),
            "Gender".to_string(),
            Some("Gender codes".to_string()),
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert_code(&ctx, &code).await.unwrap();
        let result = repo.find_code_by_id(&ctx, CodeId::new(42)).await.unwrap();
        assert_eq!(result.unwrap().name, "Gender");

        let ctx2 = make_context(2);
        let result = repo.find_code_by_id(&ctx2, CodeId::new(42)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn code_value_repository_is_tenant_isolated() {
        let repo = InMemoryCodeRepository::new();
        let code_id = CodeId::new(1);
        let code = Code::new(code_id, TenantId::new(1), "Gender".to_string(), None).unwrap();

        let cv = CodeValue::new(
            CodeValueId::new(100),
            code_id,
            TenantId::new(1),
            "Male".to_string(),
            None,
            1,
        )
        .unwrap();

        let ctx = make_context(1);
        repo.insert_code(&ctx, &code).await.unwrap();
        repo.insert_code_value(&ctx, &cv).await.unwrap();

        let result = repo
            .find_code_value_by_id(&ctx, CodeValueId::new(100))
            .await
            .unwrap();
        assert_eq!(result.unwrap().name, "Male");

        let ctx2 = make_context(2);
        let result = repo
            .find_code_value_by_id(&ctx2, CodeValueId::new(100))
            .await
            .unwrap();
        assert_eq!(result, None);
    }
}
