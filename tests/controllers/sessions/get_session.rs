use crate::common::{authenticated_request, register_authenticated_user, setup_test_server};
use serde_json::Value;

#[tokio::test]
async fn test_get_session_success() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    let response = authenticated_request(
        server.get(&format!("/api/v1/auth/sessions/{}", auth.session_id)),
        &auth,
    )
    .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    assert_eq!(
        body["response"]["data"]["session"]["id"].as_str().unwrap(),
        auth.session_id
    );
    assert_eq!(
        body["response"]["data"]["session"]["sub_sessions"][0]["activity_type"]
            .as_str()
            .unwrap(),
        "register"
    );
}
