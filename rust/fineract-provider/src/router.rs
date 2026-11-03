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
use crate::handlers::code::{
    create_code, create_code_value, delete_code, delete_code_value, get_code, get_code_value,
    list_code_values, list_codes, update_code, update_code_value,
};
use crate::handlers::collateral::{
    create_collateral, delete_collateral, get_collateral, list_collateral, update_collateral,
};
use crate::handlers::currency::{
    create_currency, delete_currency, get_currency, get_currency_by_code, list_currencies,
    update_currency,
};
use crate::handlers::datatable::{
    create_column, create_datatable, create_view, delete_column, delete_datatable, delete_view,
    get_column, get_datatable, get_view, list_columns, list_datatables, list_views, update_column,
    update_datatable, update_view,
};
use crate::handlers::document::{
    create_document, delete_document, get_document, list_documents, update_document,
};
use crate::handlers::guarantor::{
    create_guarantor, delete_guarantor, get_guarantor, list_guarantors, update_guarantor,
};
use crate::handlers::holiday::{
    create_holiday, delete_holiday, get_holiday, get_holidays_by_date, list_holidays,
    update_holiday,
};
use crate::handlers::investor::{
    create_investor, delete_investor, get_investor, list_investors, update_investor,
};
use crate::handlers::loan::{
    create_loan, delete_loan, get_loan, list_loans, record_loan_transaction, update_loan,
};
use crate::handlers::loan_origination::{
    approve_loan_application, delete_loan_application, get_loan_application,
    list_loan_applications, list_loan_applications_by_client, reject_loan_application,
    submit_loan_application, update_loan_application,
};
use crate::handlers::loan_product::{
    create_loan_product, delete_loan_product, get_loan_product, get_loan_product_by_name,
    list_loan_products, update_loan_product,
};
use crate::handlers::note::{create_note, delete_note, get_note, list_notes, update_note};
use crate::handlers::progressive_loan::{
    advance_phase, create_progressive_loan, delete_progressive_loan, get_progressive_loan,
    list_progressive_loans, list_progressive_loans_by_client, update_progressive_loan,
};
use crate::handlers::rate::{create_rate, delete_rate, get_rate, list_rates, update_rate};
use crate::handlers::report::{
    create_report, delete_report, generate_report, get_report, list_reports, update_report,
};
use crate::handlers::savings::{
    activate_savings_account, create_savings_account, delete_savings_account, get_savings_account,
    list_savings_accounts, update_savings_account,
};
use crate::handlers::savings_product::{
    create_savings_product, delete_savings_product, get_savings_product,
    get_savings_product_by_name, list_savings_products, update_savings_product,
};
use crate::handlers::staff::{
    create_staff, delete_staff, get_staff, get_staff_by_username, list_staff, update_staff,
};
use crate::handlers::tax::{create_tax, delete_tax, get_tax, list_taxes, update_tax};
use crate::handlers::working_capital_loan::{
    create_working_capital_loan, delete_working_capital_loan, get_working_capital_loan,
    list_working_capital_loans, list_working_capital_loans_by_client, update_working_capital_loan,
    update_working_capital_loan_status,
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
        // Rates
        .route("/v1/rates", get(list_rates).post(create_rate))
        .route(
            "/v1/rates/:id",
            get(get_rate).put(update_rate).delete(delete_rate),
        )
        // Taxes
        .route("/v1/taxes", get(list_taxes).post(create_tax))
        .route(
            "/v1/taxes/:id",
            get(get_tax).put(update_tax).delete(delete_tax),
        )
        // Documents
        .route("/v1/documents", get(list_documents).post(create_document))
        .route(
            "/v1/documents/:id",
            get(get_document)
                .put(update_document)
                .delete(delete_document),
        )
        // Investors
        .route("/v1/investors", get(list_investors).post(create_investor))
        .route(
            "/v1/investors/:id",
            get(get_investor)
                .put(update_investor)
                .delete(delete_investor),
        )
        // Loan Origination
        .route(
            "/v1/loan-applications",
            get(list_loan_applications).post(submit_loan_application),
        )
        .route(
            "/v1/loan-applications/:id",
            get(get_loan_application)
                .put(update_loan_application)
                .delete(delete_loan_application),
        )
        .route(
            "/v1/clients/:client_id/loan-applications",
            get(list_loan_applications_by_client),
        )
        .route(
            "/v1/loan-applications/:id/approve",
            post(approve_loan_application),
        )
        .route(
            "/v1/loan-applications/:id/reject",
            post(reject_loan_application),
        )
        // Progressive Loans
        .route(
            "/v1/progressive-loans",
            get(list_progressive_loans).post(create_progressive_loan),
        )
        .route(
            "/v1/progressive-loans/:id",
            get(get_progressive_loan)
                .put(update_progressive_loan)
                .delete(delete_progressive_loan),
        )
        .route(
            "/v1/clients/:client_id/progressive-loans",
            get(list_progressive_loans_by_client),
        )
        .route(
            "/v1/progressive-loans/:id/advance-phase",
            post(advance_phase),
        )
        // Working Capital Loans
        .route(
            "/v1/working-capital-loans",
            get(list_working_capital_loans).post(create_working_capital_loan),
        )
        .route(
            "/v1/working-capital-loans/:id",
            get(get_working_capital_loan)
                .put(update_working_capital_loan)
                .delete(delete_working_capital_loan),
        )
        .route(
            "/v1/clients/:client_id/working-capital-loans",
            get(list_working_capital_loans_by_client),
        )
        .route(
            "/v1/working-capital-loans/:id/status",
            put(update_working_capital_loan_status),
        )
        // Staff
        .route("/v1/staff", get(list_staff).post(create_staff))
        .route(
            "/v1/staff/:id",
            get(get_staff).put(update_staff).delete(delete_staff),
        )
        .route(
            "/v1/staff/by-username/:username",
            get(get_staff_by_username),
        )
        // Codes / CodeValues
        .route("/v1/codes", get(list_codes).post(create_code))
        .route(
            "/v1/codes/:id",
            get(get_code).put(update_code).delete(delete_code),
        )
        .route(
            "/v1/codes/:codeId/codeValues",
            get(list_code_values).post(create_code_value),
        )
        .route(
            "/v1/codes/:codeId/codeValues/:id",
            get(get_code_value)
                .put(update_code_value)
                .delete(delete_code_value),
        )
        // Currencies
        .route("/v1/currencies", get(list_currencies).post(create_currency))
        .route(
            "/v1/currencies/:id",
            get(get_currency)
                .put(update_currency)
                .delete(delete_currency),
        )
        .route("/v1/currencies/code/:code", get(get_currency_by_code))
        // Loan Products
        .route(
            "/v1/loanproducts",
            get(list_loan_products).post(create_loan_product),
        )
        .route(
            "/v1/loanproducts/:id",
            get(get_loan_product)
                .put(update_loan_product)
                .delete(delete_loan_product),
        )
        .route("/v1/loanproducts/name/:name", get(get_loan_product_by_name))
        // Savings Products
        .route(
            "/v1/savingsproducts",
            get(list_savings_products).post(create_savings_product),
        )
        .route(
            "/v1/savingsproducts/:id",
            get(get_savings_product)
                .put(update_savings_product)
                .delete(delete_savings_product),
        )
        .route(
            "/v1/savingsproducts/name/:name",
            get(get_savings_product_by_name),
        )
        // Notes
        .route("/v1/notes", get(list_notes).post(create_note))
        .route(
            "/v1/notes/:id",
            get(get_note).put(update_note).delete(delete_note),
        )
        // Collateral
        .route(
            "/v1/collateral",
            get(list_collateral).post(create_collateral),
        )
        .route(
            "/v1/collateral/:id",
            get(get_collateral)
                .put(update_collateral)
                .delete(delete_collateral),
        )
        // Guarantors
        .route(
            "/v1/guarantors",
            get(list_guarantors).post(create_guarantor),
        )
        .route(
            "/v1/guarantors/:id",
            get(get_guarantor)
                .put(update_guarantor)
                .delete(delete_guarantor),
        )
        // Datatables
        .route(
            "/v1/datatables",
            get(list_datatables).post(create_datatable),
        )
        .route(
            "/v1/datatables/:id",
            get(get_datatable)
                .put(update_datatable)
                .delete(delete_datatable),
        )
        .route(
            "/v1/datatables/:datatable_id/columns",
            get(list_columns).post(create_column),
        )
        .route(
            "/v1/datatables/:datatable_id/columns/:id",
            get(get_column).put(update_column).delete(delete_column),
        )
        .route(
            "/v1/datatables/:datatable_id/views",
            get(list_views).post(create_view),
        )
        .route(
            "/v1/datatables/:datatable_id/views/:id",
            get(get_view).put(update_view).delete(delete_view),
        )
        // Reports
        .route("/v1/reports", get(list_reports).post(create_report))
        .route(
            "/v1/reports/:id",
            get(get_report).put(update_report).delete(delete_report),
        )
        .route("/v1/reports/:id/generate", post(generate_report))
        // Holidays
        .route("/v1/holidays", get(list_holidays).post(create_holiday))
        .route(
            "/v1/holidays/:id",
            get(get_holiday).put(update_holiday).delete(delete_holiday),
        )
        .route("/v1/holidays/date/:date", get(get_holidays_by_date))
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
