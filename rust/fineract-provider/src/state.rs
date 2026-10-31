//! Shared application state.
//!
//! Holds the command dispatcher, authentication service, and authorization
//! service that all handlers share via Axum `State`.

use std::sync::Arc;

use fineract_command::SynchronousCommandDispatcher;
use fineract_persistence::TenantPoolManager;
use fineract_security::AuthenticationService;
use fineract_security::AuthorizationService;

/// Shared state injected into every Axum handler via `State(AppState)`.
#[derive(Clone)]
pub struct AppState {
    pub dispatcher: Arc<SynchronousCommandDispatcher>,
    pub auth_service: Arc<dyn AuthenticationService>,
    pub authz_service: Arc<dyn AuthorizationService>,
    pub pool_manager: Arc<TenantPoolManager>,
    pub connection_string: String,
}

impl AppState {
    pub fn new(
        dispatcher: SynchronousCommandDispatcher,
        auth_service: Arc<dyn AuthenticationService>,
        authz_service: Arc<dyn AuthorizationService>,
        pool_manager: Arc<TenantPoolManager>,
        connection_string: String,
    ) -> Self {
        Self {
            dispatcher: Arc::new(dispatcher),
            auth_service,
            authz_service,
            pool_manager,
            connection_string,
        }
    }
}
