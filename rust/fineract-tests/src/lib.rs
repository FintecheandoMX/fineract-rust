//! Integration test helpers.
//!
//! Spins up the Axum provider in-process against the Docker Postgres
//! database and provides a `TestApp` handle with a `reqwest::Client`.

#![forbid(unsafe_code)]

use axum::Router;
use fineract_command::SynchronousCommandDispatcher;
use fineract_persistence::TenantPoolManager;
use fineract_provider::router::router;
use fineract_provider::state::AppState;
use fineract_security::AuthenticationService;
use fineract_security::AuthorizationService;
use fineract_security::InMemoryAuthService;
use fineract_security::InMemoryAuthorizationService;
use std::net::SocketAddr;
use std::sync::Arc;

/// A running test instance of the Axum provider.
pub struct TestApp {
    pub addr: SocketAddr,
    pub client: reqwest::Client,
}

impl TestApp {
    /// Build an `AppState`, mount the router, and start serving on a random
    /// port. Returns a `TestApp` once the server is accepting connections.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dispatcher = SynchronousCommandDispatcher::new();
        let auth_service: Arc<dyn AuthenticationService> = Arc::new(InMemoryAuthService::new()?);
        let authz_service: Arc<dyn AuthorizationService> =
            Arc::new(InMemoryAuthorizationService::default());
        let pool_manager: Arc<TenantPoolManager> = Arc::new(TenantPoolManager::new());

        let connection_string = std::env::var("FINERACT_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost:5432/fineract_desarrollo".to_string()
        });

        let state = AppState::new(
            dispatcher,
            auth_service,
            authz_service,
            pool_manager,
            connection_string,
        );

        let app: Router = router(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Give the server a moment to start accepting connections.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(Self {
            addr,
            client: reqwest::Client::new(),
        })
    }

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
}
