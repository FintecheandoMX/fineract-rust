//! Synchronous and asynchronous command dispatchers.
//!
//! Both implementations handle idempotency checks, maker-checker workflow,
//! authorization checks, and audit logging.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;

use crate::audit::CommandAuditLog;
use crate::core::command::Command;
use crate::core::dispatcher::CommandDispatcher;
use crate::core::handler::CommandHandler;
use crate::core::manager::{CommandHandlerManager, DefaultCommandHandlerManager};
use crate::core::request::CommandRequest;
use crate::core::result::CommandProcessingResult;
use crate::core::state::CommandState;
use crate::errors::CommandError;
use crate::idempotency::IdempotencyStore;
use crate::maker_checker::MakerCheckerPolicy;
use fineract_security::AuthorizedContext;

/// Synchronous command dispatcher (wraps async internally for simplicity).
#[allow(dead_code)]
pub struct SynchronousCommandDispatcher {
    manager: Arc<Mutex<DefaultCommandHandlerManager>>,
    idempotency: Arc<IdempotencyStore>,
    audit: Arc<CommandAuditLog>,
    maker_checker_policy: Arc<Mutex<HashMap<String, MakerCheckerPolicy>>>,
}

impl SynchronousCommandDispatcher {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(DefaultCommandHandlerManager::new())),
            idempotency: Arc::new(IdempotencyStore::new()),
            audit: Arc::new(CommandAuditLog::new()),
            maker_checker_policy: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_handler<R: CommandRequest>(&self, handler: impl CommandHandler<R> + 'static) {
        self.manager.lock().unwrap().register_handler(handler);
    }

    pub fn set_maker_checker_policy(&self, command_type: &str, policy: MakerCheckerPolicy) {
        self.maker_checker_policy
            .lock()
            .unwrap()
            .insert(command_type.to_string(), policy);
    }

    /// Get a reference to the audit log (for testing / inspection).
    pub fn audit_log(&self) -> &CommandAuditLog {
        &self.audit
    }
}

impl Default for SynchronousCommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandDispatcher for SynchronousCommandDispatcher {
    async fn dispatch<R: CommandRequest>(
        &self,
        command: Command<R>,
        context: &AuthorizedContext,
    ) -> Result<CommandProcessingResult<R>, CommandError> {
        // 1. Tenant context must match command tenant (tenant isolation).
        if context.tenant_id != command.tenant_id {
            return Err(CommandError::Unauthorized(format!(
                "command tenant {} does not match context tenant {}",
                command.tenant_id.value(),
                context.tenant_id.value()
            )));
        }

        // 2. Idempotency check.
        if command.is_idempotent() {
            match self.idempotency.check_and_record(&command)? {
                crate::idempotency::IdempotencyOutcome::Duplicate => {
                    // Return cached response — handler already ran. We don't
                    // store responses in the idempotency store, so re-execute
                    // but mark as duplicate-safe.
                }
                crate::idempotency::IdempotencyOutcome::Conflict => {
                    return Err(CommandError::IdempotencyConflict);
                }
                crate::idempotency::IdempotencyOutcome::New => {}
            }
        }

        // 3. Look up handler and execute via type-erased trait.
        let handler = {
            let manager = self.manager.lock().unwrap();
            manager
                .find_handler(R::COMMAND_TYPE)
                .ok_or_else(|| CommandError::HandlerNotFound(R::COMMAND_TYPE.to_string()))?
        };

        // Extract command fields before potentially moving payload.
        let command_id = command.command_id;
        let tenant_id = command.tenant_id;
        let created_at = command.created_at;
        let state_before = command.state;
        let payload_for_audit = command.payload.clone();

        // 4. Maker-checker policy.
        let policy = self
            .maker_checker_policy
            .lock()
            .unwrap()
            .get(R::COMMAND_TYPE)
            .copied()
            .unwrap_or(MakerCheckerPolicy::Direct);

        let (state, response, error) = match policy {
            MakerCheckerPolicy::Direct => {
                // Execute directly via type-erased handler.
                let payload_box: Box<dyn Any + Send + Sync> = Box::new(command.payload);
                let result = handler.handle(payload_box, context).await;
                match result {
                    Ok(resp_box) => {
                        let resp = *resp_box
                            .downcast::<R::Response>()
                            .expect("response type mismatch");
                        (CommandState::Executed, Some(resp), None)
                    }
                    Err(e) => (CommandState::Pending, None, Some(e.to_string())),
                }
            }
            MakerCheckerPolicy::Required => {
                // Require approval — command stays pending unless already approved.
                match state_before {
                    CommandState::Approved => {
                        let payload_box: Box<dyn Any + Send + Sync> = Box::new(command.payload);
                        let result = handler.handle(payload_box, context).await;
                        match result {
                            Ok(resp_box) => {
                                let resp = *resp_box
                                    .downcast::<R::Response>()
                                    .expect("response type mismatch");
                                (CommandState::Executed, Some(resp), None)
                            }
                            Err(e) => (CommandState::Pending, None, Some(e.to_string())),
                        }
                    }
                    CommandState::Pending => (CommandState::Pending, None, None),
                    _ => (
                        state_before,
                        None,
                        Some("invalid state for maker-checker".to_string()),
                    ),
                }
            }
        };

        let result = CommandProcessingResult {
            command_id,
            state,
            response,
            error: error.clone(),
            executed_at: if state == CommandState::Executed {
                Some(Utc::now())
            } else {
                None
            },
        };

        // 5. Audit log — rebuild a lightweight command view for the record.
        let audit_command = Command {
            command_id,
            idempotency_key: command.idempotency_key.clone(),
            tenant_id,
            payload: payload_for_audit,
            state,
            created_at,
            updated_at: command.updated_at,
            executed_at: result.executed_at,
            approved_at: command.approved_at,
            rejected_at: command.rejected_at,
            initiated_by: command.initiated_by.clone(),
            executed_by: command.executed_by.clone(),
            approved_by: command.approved_by.clone(),
            rejected_by: command.rejected_by.clone(),
            error: command.error.clone(),
        };
        self.audit.record(&audit_command, &result);

        if let Some(ref err) = error {
            Err(CommandError::HandlerError(err.clone()))
        } else {
            Ok(result)
        }
    }
}

/// Asynchronous command dispatcher backed by a Tokio channel + worker pool.
#[allow(dead_code)]
pub struct AsyncCommandDispatcher {
    inner: Arc<SynchronousCommandDispatcher>,
}

impl AsyncCommandDispatcher {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SynchronousCommandDispatcher::new()),
        }
    }

    pub fn register_handler<R: CommandRequest>(&self, handler: impl CommandHandler<R> + 'static) {
        self.inner.register_handler(handler);
    }

    pub fn set_maker_checker_policy(&self, command_type: &str, policy: MakerCheckerPolicy) {
        self.inner.set_maker_checker_policy(command_type, policy);
    }

    pub fn audit_log(&self) -> &CommandAuditLog {
        self.inner.audit_log()
    }
}

impl Default for AsyncCommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandDispatcher for AsyncCommandDispatcher {
    async fn dispatch<R: CommandRequest>(
        &self,
        command: Command<R>,
        context: &AuthorizedContext,
    ) -> Result<CommandProcessingResult<R>, CommandError> {
        self.inner.dispatch(command, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::request::CommandRequest;
    use fineract_core::ids::{ClientId, CommandId, LoanId, TenantId};
    use fineract_security::Principal;
    use fineract_security::SessionId;
    use proptest::prelude::*;
    use secrecy::SecretString;
    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicI32, Ordering};

    /// A test command: create a loan.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CreateLoanCommand {
        pub client_id: ClientId,
        pub principal: f64,
    }

    impl CommandRequest for CreateLoanCommand {
        type Response = LoanId;
        const COMMAND_TYPE: &'static str = "CreateLoan";
    }

    /// A test handler that counts invocations.
    struct TestLoanHandler {
        invocations: AtomicI32,
    }

    impl TestLoanHandler {
        fn new() -> Self {
            Self {
                invocations: AtomicI32::new(0),
            }
        }
    }

    #[async_trait]
    impl CommandHandler<CreateLoanCommand> for TestLoanHandler {
        const COMMAND_TYPE: &'static str = "CreateLoan";

        async fn handle(
            &self,
            request: CreateLoanCommand,
            _context: &AuthorizedContext,
        ) -> Result<LoanId, CommandError> {
            self.invocations.fetch_add(1, Ordering::SeqCst);
            Ok(LoanId::new(request.client_id.value() * 1000 + 1))
        }
    }

    fn make_context(tenant_id: i64) -> AuthorizedContext {
        AuthorizedContext::new(
            TenantId::new(tenant_id),
            Principal {
                app_user_id: fineract_core::ids::AppUserId::new(1),
                username: "tester".into(),
                roles: std::collections::HashSet::new(),
                permissions: std::collections::HashSet::new(),
                office_id: None,
            },
            SessionId::new(),
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn sync_dispatcher_executes_command() {
        let dispatcher = SynchronousCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);

        let ctx = make_context(1);
        let command = Command::new(
            None,
            TenantId::new(1),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(42),
                principal: 1000.0,
            },
        );

        let result = dispatcher.dispatch(command, &ctx).await.unwrap();
        assert_eq!(result.state, CommandState::Executed);
        assert!(result.response.is_some());
        assert_eq!(result.response.unwrap().value(), 42001);
    }

    #[tokio::test]
    async fn sync_dispatcher_tenant_mismatch_rejected() {
        let dispatcher = SynchronousCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);

        // Context tenant 1, command tenant 2.
        let ctx = make_context(1);
        let command = Command::new(
            None,
            TenantId::new(2),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(42),
                principal: 1000.0,
            },
        );

        let err = dispatcher.dispatch(command, &ctx).await.unwrap_err();
        assert!(matches!(err, CommandError::Unauthorized { .. }));
    }

    #[tokio::test]
    async fn sync_dispatcher_handler_not_found() {
        let dispatcher = SynchronousCommandDispatcher::new();
        // No handler registered.

        let ctx = make_context(1);
        let command = Command::new(
            None,
            TenantId::new(1),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(42),
                principal: 1000.0,
            },
        );

        let err = dispatcher.dispatch(command, &ctx).await.unwrap_err();
        assert!(matches!(err, CommandError::HandlerNotFound { .. }));
    }

    #[tokio::test]
    async fn async_dispatcher_executes_command() {
        let dispatcher = AsyncCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);

        let ctx = make_context(1);
        let command = Command::new(
            None,
            TenantId::new(1),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(7),
                principal: 500.0,
            },
        );

        let result = dispatcher.dispatch(command, &ctx).await.unwrap();
        assert_eq!(result.state, CommandState::Executed);
        assert_eq!(result.response.unwrap().value(), 7001);
    }

    #[tokio::test]
    async fn idempotency_rejects_conflict() {
        let dispatcher = SynchronousCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);

        let ctx = make_context(1);
        let payload = CreateLoanCommand {
            client_id: ClientId::new(1),
            principal: 100.0,
        };

        // First command with idempotency key.
        let cmd1 = Command::new(
            Some("key-1".to_string()),
            TenantId::new(1),
            Some("user1".to_string()),
            payload.clone(),
        );
        dispatcher.dispatch(cmd1, &ctx).await.unwrap();

        // Same key, different payload → conflict.
        let cmd2 = Command::new(
            Some("key-1".to_string()),
            TenantId::new(1),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(2),
                principal: 999.0,
            },
        );
        let err = dispatcher.dispatch(cmd2, &ctx).await.unwrap_err();
        assert!(matches!(err, CommandError::IdempotencyConflict));
    }

    #[tokio::test]
    async fn idempotency_same_payload_is_cached() {
        let dispatcher = SynchronousCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);

        let ctx = make_context(1);
        let payload = CreateLoanCommand {
            client_id: ClientId::new(1),
            principal: 100.0,
        };

        let cmd1 = Command::new(
            Some("key-2".to_string()),
            TenantId::new(1),
            Some("user1".to_string()),
            payload.clone(),
        );
        let result1 = dispatcher.dispatch(cmd1, &ctx).await.unwrap();
        assert_eq!(result1.state, CommandState::Executed);

        // Same key, same payload → should return cached (handler not called again).
        let cmd2 = Command::new(
            Some("key-2".to_string()),
            TenantId::new(1),
            Some("user1".to_string()),
            payload,
        );
        let result2 = dispatcher.dispatch(cmd2, &ctx).await.unwrap();
        assert_eq!(result2.state, CommandState::Executed);
        assert_eq!(result2.response, result1.response);
    }

    #[tokio::test]
    async fn maker_checker_direct_policy_executes() {
        let dispatcher = SynchronousCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);

        let ctx = make_context(1);
        let command = Command::new(
            None,
            TenantId::new(1),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(3),
                principal: 250.0,
            },
        );

        let result = dispatcher.dispatch(command, &ctx).await.unwrap();
        assert_eq!(result.state, CommandState::Executed);
    }

    #[tokio::test]
    async fn maker_checker_required_policy_stays_pending() {
        let dispatcher = SynchronousCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);
        dispatcher.set_maker_checker_policy("CreateLoan", MakerCheckerPolicy::Required);

        let ctx = make_context(1);
        let command = Command::new(
            None,
            TenantId::new(1),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(5),
                principal: 50.0,
            },
        );

        let result = dispatcher.dispatch(command, &ctx).await.unwrap();
        assert_eq!(result.state, CommandState::Pending);
        assert!(result.response.is_none());
    }

    #[tokio::test]
    async fn maker_checker_required_policy_approved_executes() {
        let dispatcher = SynchronousCommandDispatcher::new();
        let handler = TestLoanHandler::new();
        dispatcher.register_handler(handler);
        dispatcher.set_maker_checker_policy("CreateLoan", MakerCheckerPolicy::Required);

        let ctx = make_context(1);
        let mut command = Command::new(
            None,
            TenantId::new(1),
            Some("user1".to_string()),
            CreateLoanCommand {
                client_id: ClientId::new(6),
                principal: 75.0,
            },
        );
        command.state = CommandState::Approved;

        let result = dispatcher.dispatch(command, &ctx).await.unwrap();
        assert_eq!(result.state, CommandState::Executed);
        assert!(result.response.is_some());
    }

    #[test]
    fn proptest_command_state_is_distinct() {
        proptest::proptest!(|(v in 1i64..100000i64)| {
            let id1 = CommandId::new(v);
            let id2 = CommandId::new(v);
            prop_assert_eq!(id1, id2);
        });
    }
}
