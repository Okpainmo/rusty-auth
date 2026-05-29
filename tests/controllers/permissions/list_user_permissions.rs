use crate::common::{authenticated_request, register_authenticated_user, setup_test_server_and_db};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_list_user_permissions_success() {
    let (server, db) = setup_test_server_and_db().await;
    let auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let permission_name = format!("admin_permission_{}", unique_id);

    let permission_response = authenticated_request(server.post("/api/v1/auth/permissions"), &auth)
        .json(&serde_json::json!({
            "name": permission_name,
            "description": "Admin permission"
        }))
        .await;
    permission_response.assert_status(axum::http::StatusCode::CREATED);
    let permission_body = permission_response.json::<Value>();
    let permission_id = permission_body["response"]["data"]["id"].as_str().unwrap();

    let user_role_id: String = sqlx::query_scalar("SELECT id::text FROM roles WHERE name = 'user'")
        .fetch_one(&db)
        .await
        .unwrap();

    authenticated_request(server.post("/api/v1/auth/roles/permissions"), &auth)
        .json(&serde_json::json!({
            "role_id": user_role_id,
            "permission_id": permission_id
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    let response = authenticated_request(
        server.get(&format!("/api/v1/auth/permissions/user/{}", auth.user_id)),
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
            .any(|permission| permission["id"].as_str() == Some(permission_id))
    );
}
