//! Shared application state.
//!
//! Holds the command dispatcher, authentication service, authorization
//! service, tenant repositories (for read operations), and the connection
//! string that all handlers share via Axum `State`.

use std::sync::Arc;

use fineract_accounting::SqlxAccountRepository;
use fineract_branch::SqlxBranchRepository;
use fineract_charge::SqlxChargeRepository;
use fineract_client::SqlxClientRepository;
use fineract_command::SynchronousCommandDispatcher;
use fineract_loan::SqlxLoanRepository;
use fineract_persistence::TenantPoolManager;
use fineract_savings::SqlxSavingsAccountRepository;
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
    pub office_repository: Arc<SqlxBranchRepository>,
    pub loan_repository: Arc<SqlxLoanRepository>,
    pub savings_repository: Arc<SqlxSavingsAccountRepository>,
    pub account_repository: Arc<SqlxAccountRepository>,
    pub client_repository: Arc<SqlxClientRepository>,
    pub charge_repository: Arc<SqlxChargeRepository>,
}

impl AppState {
    pub fn new(
        dispatcher: SynchronousCommandDispatcher,
        auth_service: Arc<dyn AuthenticationService>,
        authz_service: Arc<dyn AuthorizationService>,
        pool_manager: Arc<TenantPoolManager>,
        connection_string: String,
    ) -> Self {
        let cs1 = connection_string.clone();
        let cs2 = connection_string.clone();
        let cs3 = connection_string.clone();
        let cs4 = connection_string.clone();
        let cs5 = connection_string.clone();
        let cs6 = connection_string.clone();
        let pm1 = pool_manager.clone();
        let pm2 = pool_manager.clone();
        let pm3 = pool_manager.clone();
        let pm4 = pool_manager.clone();
        let pm5 = pool_manager.clone();
        let pm6 = pool_manager.clone();
        Self {
            dispatcher: Arc::new(dispatcher),
            auth_service,
            authz_service,
            pool_manager: pool_manager.clone(),
            connection_string,
            office_repository: Arc::new(SqlxBranchRepository::new(pm1, cs1)),
            loan_repository: Arc::new(SqlxLoanRepository::new(pm2, cs2)),
            savings_repository: Arc::new(SqlxSavingsAccountRepository::new(pm3, cs3)),
            account_repository: Arc::new(SqlxAccountRepository::new(pm4, cs4)),
            client_repository: Arc::new(SqlxClientRepository::new(pm5, cs5)),
            charge_repository: Arc::new(SqlxChargeRepository::new(pm6, cs6)),
        }
    }
}
