//! Loan API integration tests.
//!
//! Verifies that protected routes require authentication and tenant header.

use fineract_tests::TestApp;

#[tokio::test]
async fn loans_without_tenant_header_returns_400() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/v1/loans", app.base_url());

    let resp = app
        .client
        .get(&url)
        .send()
        .await
        .expect("request to succeed");

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn loans_with_tenant_but_no_auth_returns_401() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/v1/loans", app.base_url());

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
async fn create_loan_with_tenant_but_no_auth_returns_401() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/v1/loans", app.base_url());

    let resp = app
        .client
        .post(&url)
        .header("Fineract-Platform-TenantId", "1")
        .json(&serde_json::json!({
            "client_id": 1,
            "loan_type": "Individual",
            "principal": "1000.00",
            "currency": "USD",
            "interest_rate": "5.0",
            "term_count": 12,
            "term_period": "Months",
            "disbursement_date": "2024-01-01"
        }))
        .send()
        .await
        .expect("request to succeed");

    assert_eq!(resp.status(), 401);
}
