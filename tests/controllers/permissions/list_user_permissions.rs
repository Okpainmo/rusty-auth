use crate::common::{RegisterRequest, TestRegisterResponse, setup_test_server_and_db};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_list_user_permissions_success() {
    let (server, db) = setup_test_server_and_db().await;

    let unique_id = Uuid::new_v4().to_string();
    let permission_name = format!("admin_permission_{}", unique_id);

    let register_response = server
        .post("/api/v1/auth/register/admin")
        .json(&RegisterRequest {
            first_name: "Rbac".to_string(),
            last_name: "Admin".to_string(),
            email: format!("rbac_permissions_{}@example.com", unique_id),
            password: "password123".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(unique_id[0..10].to_string()),
        })
        .await;
    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body = register_response.json::<TestRegisterResponse>();
    let user_id = register_body.response.unwrap().user_profile.unwrap().id;

    let permission_response = server
        .post("/api/v1/auth/permissions")
        .json(&serde_json::json!({
            "name": permission_name,
            "description": "Admin permission"
        }))
        .await;
    permission_response.assert_status(axum::http::StatusCode::CREATED);
    let permission_body = permission_response.json::<Value>();
    let permission_id = permission_body["response"]["data"]["id"].as_str().unwrap();

    let admin_role_id: String =
        sqlx::query_scalar("SELECT id::text FROM roles WHERE name = 'admin'")
            .fetch_one(&db)
            .await
            .unwrap();

    server
        .post("/api/v1/auth/roles/permissions")
        .json(&serde_json::json!({
            "role_id": admin_role_id,
            "permission_id": permission_id
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    let response = server
        .get(&format!("/api/v1/auth/permissions/user/{}", user_id))
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
