use crate::common::{RegisterRequest, TestRegisterResponse, setup_test_server_and_db};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_assign_user_role_success() {
    let (server, db) = setup_test_server_and_db().await;

    let unique_id = Uuid::new_v4().to_string();
    let role_name = format!("assignable_role_{}", unique_id);

    let register_response = server
        .post("/api/v1/auth/register/admin")
        .json(&RegisterRequest {
            first_name: "Rbac".to_string(),
            last_name: "Admin".to_string(),
            email: format!("rbac_admin_{}@example.com", unique_id),
            password: "password123".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(unique_id[0..10].to_string()),
        })
        .await;
    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body = register_response.json::<TestRegisterResponse>();
    let user_id = register_body.response.unwrap().user_profile.unwrap().id;

    let role_response = server
        .post("/api/v1/auth/roles")
        .json(&serde_json::json!({
            "name": role_name,
            "description": "Assignable role"
        }))
        .await;
    role_response.assert_status(axum::http::StatusCode::CREATED);
    let role_body = role_response.json::<Value>();
    let role_id = role_body["response"]["data"]["id"].as_str().unwrap();

    server
        .post("/api/v1/auth/roles/user/assign")
        .json(&serde_json::json!({
            "user_id": user_id,
            "role_id": role_id
        }))
        .await
        .assert_status(axum::http::StatusCode::OK);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = $1 AND role_id = $2::uuid",
    )
    .bind(user_id)
    .bind(role_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(count, 1);
}
