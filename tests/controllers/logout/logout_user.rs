use crate::common::{
    RegisterRequest, TestLoginResponse, TestRegisterResponse, setup_test_server,
    setup_test_server_and_db,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct LogoutRequest {
    user_email: String,
    session_id: String,
}

#[tokio::test]
async fn test_logout_user_success() {
    let (server, db) = setup_test_server_and_db().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("logout_{}@example.com", unique_id);

    let register_response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Logout".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: "password123".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(unique_id[0..10].to_string()),
        })
        .await;

    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body = register_response.json::<TestRegisterResponse>();
    let session_id = register_body.response.unwrap().session_id;

    let response = server
        .post("/api/v1/auth/logout")
        .json(&LogoutRequest {
            user_email: email.clone(),
            session_id: session_id.clone(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<TestLoginResponse>();
    assert_eq!(body.response_message, "Logout successful");

    let revoked_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE id = $1::uuid AND status = 'revoked' AND revoked_at IS NOT NULL",
    )
    .bind(&session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(revoked_count, 1);

    let sub_session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'logout'",
    )
    .bind(&session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(sub_session_count, 1);

    let request_path: String = sqlx::query_scalar(
        "SELECT request_path FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'logout'",
    )
    .bind(&session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_path, "/api/v1/auth/logout");

    let request_method: String = sqlx::query_scalar(
        "SELECT request_method FROM sub_sessions WHERE session_id = $1::uuid AND activity_type = 'logout'",
    )
    .bind(&session_id)
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(request_method, "POST");
}

#[tokio::test]
async fn test_logout_non_existent_user() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/v1/auth/logout")
        .json(&LogoutRequest {
            user_email: "ghost@example.com".to_string(),
            session_id: Uuid::new_v4().to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}
