//! Health endpoint integration tests.

use fineract_tests::TestApp;

#[tokio::test]
async fn healthz_returns_ok() {
    let app = TestApp::new().await.expect("test app to start");
    let url = format!("{}/healthz", app.base_url());

    let resp = app
        .client
        .get(&url)
        .send()
        .await
        .expect("request to succeed");
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.expect("response body");
    assert_eq!(body, "ok");
}
