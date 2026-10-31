//! Main entry point for the Fineract Rust REST API provider.
//!
//! Builds the command dispatcher, registers all module command handlers,
//! constructs the Axum router, and binds the TCP listener.

#![forbid(unsafe_code)]
#![allow(async_fn_in_trait)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::serve;
use fineract_command::SynchronousCommandDispatcher;
use fineract_persistence::TenantPoolManager;
use fineract_security::AuthorizationService;
use fineract_security::InMemoryAuthorizationService;
use fineract_security::{AuthenticationService, InMemoryAuthService};
use secrecy::SecretString;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use fineract_provider::router::router;
use fineract_provider::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fineract_provider=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // --- Command dispatcher ---
    let dispatcher = SynchronousCommandDispatcher::new();

    // --- Authentication service ---
    let auth_service: Arc<dyn AuthenticationService> = Arc::new(InMemoryAuthService::new()?);

    // --- Authorization service ---
    let authz_service: Arc<dyn AuthorizationService> =
        Arc::new(InMemoryAuthorizationService::default());

    // --- Tenant pool manager ---
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

    let app = router(state);

    let addr: SocketAddr = "0.0.0.0:8080".parse().expect("valid socket address");
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}
