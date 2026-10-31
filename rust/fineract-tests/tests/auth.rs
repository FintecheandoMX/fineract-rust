//! Authentication integration tests.

use fineract_tests::TestApp;

#[tokio::test]
async fn login_without_tenant_header_returns_400() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/auth/login", app.base_url());

    let resp = app
        .client
        .post(&url)
        .json(&serde_json::json!({
            "username": "admin",
            "password": "password"
        }))
        .send()
        .await
        .expect("request to succeed");

    // /auth/login now requires Fineract-Platform-TenantId header
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn login_with_invalid_credentials_returns_401() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/auth/login", app.base_url());

    let resp = app
        .client
        .post(&url)
        .header("Fineract-Platform-TenantId", "1")
        .json(&serde_json::json!({
            "username": "admin",
            "password": "wrong"
        }))
        .send()
        .await
        .expect("request to succeed");

    assert_eq!(resp.status(), 401);
}
