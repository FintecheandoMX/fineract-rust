//! Code and CodeValue domain service with authorization enforcement.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{Code, CodeError, CodeValue};
use crate::repository::CodeRepository;
use fineract_core::ids::CodeId;
use fineract_core::ids::CodeValueId;
use fineract_security::AuthorizationService;
use fineract_security::AuthorizedContext;
use fineract_security::Permission;

pub type CodeServiceResult<T> = Result<T, CodeError>;

#[async_trait]
pub trait CodeService: Send + Sync {
    async fn create_code(&self, context: &AuthorizedContext, code: Code)
        -> CodeServiceResult<Code>;
    async fn get_code(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeServiceResult<Option<Code>>;
    async fn list_codes(&self, context: &AuthorizedContext) -> CodeServiceResult<Vec<Code>>;
    async fn update_code(&self, context: &AuthorizedContext, code: Code)
        -> CodeServiceResult<Code>;
    async fn delete_code(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeServiceResult<CodeId>;
    async fn create_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: CodeValue,
    ) -> CodeServiceResult<CodeValue>;
    async fn get_code_value(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeServiceResult<Option<CodeValue>>;
    async fn list_code_values_by_code_id(
        &self,
        context: &AuthorizedContext,
        code_id: CodeId,
    ) -> CodeServiceResult<Vec<CodeValue>>;
    async fn update_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: CodeValue,
    ) -> CodeServiceResult<CodeValue>;
    async fn delete_code_value(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeServiceResult<CodeValueId>;
}

pub struct CodeServiceImpl<R: CodeRepository> {
    repository: R,
    auth_service: Arc<dyn AuthorizationService>,
}

impl<R: CodeRepository> CodeServiceImpl<R> {
    pub fn new(repository: R, auth_service: Arc<dyn AuthorizationService>) -> Self {
        Self {
            repository,
            auth_service,
        }
    }
}

#[async_trait]
impl<R: CodeRepository> CodeService for CodeServiceImpl<R> {
    async fn create_code(
        &self,
        context: &AuthorizedContext,
        code: Code,
    ) -> CodeServiceResult<Code> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        if code.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        self.repository.insert_code(context, &code).await?;
        Ok(code)
    }

    async fn get_code(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeServiceResult<Option<Code>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        let code = self.repository.find_code_by_id(context, id).await?;

        if let Some(ref c) = code {
            if c.tenant_id != context.tenant_id {
                return Err(CodeError::TenantIsolationViolation);
            }
        }

        Ok(code)
    }

    async fn list_codes(&self, context: &AuthorizedContext) -> CodeServiceResult<Vec<Code>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        self.repository.find_all_codes(context).await
    }

    async fn update_code(
        &self,
        context: &AuthorizedContext,
        code: Code,
    ) -> CodeServiceResult<Code> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        if code.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_code_by_id(context, code.id)
            .await?
            .ok_or(CodeError::NotFound(code.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        self.repository.update_code(context, &code).await?;
        Ok(code)
    }

    async fn delete_code(
        &self,
        context: &AuthorizedContext,
        id: CodeId,
    ) -> CodeServiceResult<CodeId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        let code = self
            .repository
            .find_code_by_id(context, id)
            .await?
            .ok_or(CodeError::NotFound(id))?;

        if code.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        self.repository.delete_code(context, id).await?;
        Ok(id)
    }

    async fn create_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: CodeValue,
    ) -> CodeServiceResult<CodeValue> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        if code_value.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        self.repository
            .insert_code_value(context, &code_value)
            .await?;
        Ok(code_value)
    }

    async fn get_code_value(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeServiceResult<Option<CodeValue>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        let cv = self.repository.find_code_value_by_id(context, id).await?;

        if let Some(ref c) = cv {
            if c.tenant_id != context.tenant_id {
                return Err(CodeError::TenantIsolationViolation);
            }
        }

        Ok(cv)
    }

    async fn list_code_values_by_code_id(
        &self,
        context: &AuthorizedContext,
        code_id: CodeId,
    ) -> CodeServiceResult<Vec<CodeValue>> {
        self.auth_service
            .check(context, Permission::view_client())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        self.repository
            .find_code_values_by_code_id(context, code_id)
            .await
    }

    async fn update_code_value(
        &self,
        context: &AuthorizedContext,
        code_value: CodeValue,
    ) -> CodeServiceResult<CodeValue> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        if code_value.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        let existing = self
            .repository
            .find_code_value_by_id(context, code_value.id)
            .await?
            .ok_or(CodeError::CodeValueNotFound(code_value.id))?;

        if existing.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        self.repository
            .update_code_value(context, &code_value)
            .await?;
        Ok(code_value)
    }

    async fn delete_code_value(
        &self,
        context: &AuthorizedContext,
        id: CodeValueId,
    ) -> CodeServiceResult<CodeValueId> {
        self.auth_service
            .check(context, Permission::manage_tenants())
            .await
            .map_err(|e| CodeError::Internal {
                detail: e.to_string(),
            })?;

        let cv = self
            .repository
            .find_code_value_by_id(context, id)
            .await?
            .ok_or(CodeError::CodeValueNotFound(id))?;

        if cv.tenant_id != context.tenant_id {
            return Err(CodeError::TenantIsolationViolation);
        }

        self.repository.delete_code_value(context, id).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn CodeService) {}
    }
}
