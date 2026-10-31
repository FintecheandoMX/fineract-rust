//! HTTP route definitions.
//!
//! All API routes (except /auth/*) are protected by the auth middleware,
//! which extracts the tenant ID and bearer token, authenticates the caller,
//! and injects an `AuthorizedContext` into request extensions.

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::auth::{auth_middleware, AuthContext};
use crate::handlers::account::{
    create_account, delete_account, get_account, list_accounts, update_account,
};
use crate::handlers::auth::{healthz, login, logout, refresh_token};
use crate::handlers::branch::{
    create_office, delete_office, get_office, list_offices, update_office,
};
use crate::handlers::charge::{
    create_charge, delete_charge, get_charge, list_charges, update_charge,
};
use crate::handlers::client::{
    activate_client, create_client, create_group, delete_client, delete_group, get_client,
    get_group, list_clients, list_groups, update_client, update_group,
};
use crate::handlers::loan::{
    create_loan, delete_loan, get_loan, list_loans, record_loan_transaction, update_loan,
};
use crate::handlers::savings::{
    activate_savings_account, create_savings_account, delete_savings_account, get_savings_account,
    list_savings_accounts, update_savings_account,
};
use crate::state::AppState;

/// Build the Axum `Router` with all API routes.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        // Offices / branches
        .route("/v1/offices", get(list_offices).post(create_office))
        .route(
            "/v1/offices/:id",
            get(get_office).put(update_office).delete(delete_office),
        )
        // Loans
        .route("/v1/loans", get(list_loans).post(create_loan))
        .route(
            "/v1/loans/:id",
            get(get_loan).put(update_loan).delete(delete_loan),
        )
        .route("/v1/loans/:id/transactions", post(record_loan_transaction))
        // Savings accounts
        .route(
            "/v1/savings-accounts",
            get(list_savings_accounts).post(create_savings_account),
        )
        .route(
            "/v1/savings-accounts/:id",
            get(get_savings_account)
                .put(update_savings_account)
                .delete(delete_savings_account),
        )
        .route(
            "/v1/savings-accounts/:id/activate",
            post(activate_savings_account),
        )
        // Accounts
        .route("/v1/accounts", get(list_accounts).post(create_account))
        .route(
            "/v1/accounts/:id",
            get(get_account).put(update_account).delete(delete_account),
        )
        // Clients
        .route("/v1/clients", get(list_clients).post(create_client))
        .route(
            "/v1/clients/:id",
            get(get_client).put(update_client).delete(delete_client),
        )
        .route("/v1/clients/:id/activate", post(activate_client))
        // Groups
        .route("/v1/groups", get(list_groups).post(create_group))
        .route(
            "/v1/groups/:id",
            get(get_group).put(update_group).delete(delete_group),
        )
        // Charges
        .route("/v1/charges", get(list_charges).post(create_charge))
        .route(
            "/v1/charges/:id",
            get(get_charge).put(update_charge).delete(delete_charge),
        )
        // Protect all API routes with auth middleware
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        // Health (no auth required)
        .route("/healthz", get(healthz))
        // Authentication (no auth middleware — these issue tokens)
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/logout", post(logout))
        // Nest API under / (auth middleware applies to all /v1/* routes)
        .merge(api)
        .with_state(state)
}
