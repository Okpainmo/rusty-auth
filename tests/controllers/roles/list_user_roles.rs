use crate::common::{authenticated_request, register_authenticated_user, setup_test_server};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_list_user_roles_success() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let role_name = format!("listed_role_{}", unique_id);

    let role_response = authenticated_request(server.post("/api/v1/auth/roles"), &auth)
        .json(&serde_json::json!({
            "name": role_name,
            "description": "Listed role"
        }))
        .await;
    role_response.assert_status(axum::http::StatusCode::CREATED);
    let role_body = role_response.json::<Value>();
    let role_id = role_body["response"]["data"]["id"].as_str().unwrap();

    authenticated_request(server.post("/api/v1/auth/roles/user/assign"), &auth)
        .json(&serde_json::json!({
            "user_id": auth.user_id,
            "role_id": role_id
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    let response = authenticated_request(
        server.get(&format!("/api/v1/auth/roles/user/{}", auth.user_id)),
        &auth,
    )
    .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    assert!(
        body["response"]["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|role| role["name"].as_str() == Some(role_name.as_str()))
    );
}
