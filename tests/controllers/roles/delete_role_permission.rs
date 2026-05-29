use crate::common::{authenticated_request, register_authenticated_user, setup_test_server_and_db};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_delete_role_permission_success() {
    let (server, db) = setup_test_server_and_db().await;
    let auth = register_authenticated_user(&server).await;

    let unique_id = Uuid::new_v4().to_string();
    let role_name = format!("delete_role_permission_role_{}", unique_id);
    let permission_name = format!("delete_role_permission_permission_{}", unique_id);

    let role_response = authenticated_request(server.post("/api/v1/auth/roles"), &auth)
        .json(&serde_json::json!({
            "name": role_name,
            "description": "Role permission deletion test role"
        }))
        .await;
    role_response.assert_status(axum::http::StatusCode::CREATED);
    let role_body = role_response.json::<Value>();
    let role_id = role_body["response"]["data"]["id"].as_str().unwrap();

    let permission_response = authenticated_request(server.post("/api/v1/auth/permissions"), &auth)
        .json(&serde_json::json!({
            "name": permission_name,
            "description": "Role permission deletion test permission"
        }))
        .await;
    permission_response.assert_status(axum::http::StatusCode::CREATED);
    let permission_body = permission_response.json::<Value>();
    let permission_id = permission_body["response"]["data"]["id"].as_str().unwrap();

    authenticated_request(server.post("/api/v1/auth/roles/permissions"), &auth)
        .json(&serde_json::json!({
            "role_id": role_id,
            "permission_id": permission_id
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    let response = authenticated_request(server.delete("/api/v1/auth/roles/permissions"), &auth)
        .json(&serde_json::json!({
            "role_id": role_id,
            "permission_id": permission_id
        }))
        .await;

    response.assert_status(axum::http::StatusCode::OK);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM role_permissions WHERE role_id = $1::uuid AND permission_id = $2::uuid",
    )
    .bind(role_id)
    .bind(permission_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(count, 0);
}
