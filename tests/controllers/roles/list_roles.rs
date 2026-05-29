use crate::common::setup_test_server;

#[tokio::test]
async fn test_list_roles_success() {
    let server = setup_test_server().await;

    server
        .get("/api/v1/auth/roles")
        .await
        .assert_status(axum::http::StatusCode::OK);
}
