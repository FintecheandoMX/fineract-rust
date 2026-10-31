//! Savings account API integration tests.
//!
//! Verifies that protected routes require authentication and tenant header.

use fineract_tests::TestApp;

#[tokio::test]
async fn savings_accounts_without_tenant_header_returns_400() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/v1/savings-accounts", app.base_url());

    let resp = app
        .client
        .get(&url)
        .send()
        .await
        .expect("request to succeed");

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn savings_accounts_with_tenant_but_no_auth_returns_401() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/v1/savings-accounts", app.base_url());

    let resp = app
        .client
        .get(&url)
        .header("Fineract-Platform-TenantId", "1")
        .send()
        .await
        .expect("request to succeed");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn create_savings_account_with_tenant_but_no_auth_returns_401() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/v1/savings-accounts", app.base_url());

    let resp = app
        .client
        .post(&url)
        .header("Fineract-Platform-TenantId", "1")
        .json(&serde_json::json!({
            "account_no": "SA-001",
            "account_type": "Savings",
            "currency": "USD",
            "active": true
        }))
        .send()
        .await
        .expect("request to succeed");

    assert_eq!(resp.status(), 401);
}
