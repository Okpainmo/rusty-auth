use crate::common::{RegisterRequest, TestRegisterResponse, setup_test_server};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_list_user_sessions_success() {
    let server = setup_test_server().await;

    let unique_id = Uuid::new_v4().to_string();
    let register_response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Session".to_string(),
            last_name: "User".to_string(),
            email: format!("session_user_list_{}@example.com", unique_id),
            password: "password123".to_string(),
            country: Some("TestCountry".to_string()),
            country_code: Some("TC".to_string()),
            phone_number: Some(unique_id[0..10].to_string()),
        })
        .await;

    register_response.assert_status(axum::http::StatusCode::CREATED);
    let register_body = register_response.json::<TestRegisterResponse>();
    let register_response = register_body.response.unwrap();
    let user_id = register_response.user_profile.unwrap().id;
    let session_id = register_response.session_id;

    let response = server
        .get(&format!("/api/v1/auth/sessions/users/{}", user_id))
        .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    let sessions = body["response"]["data"].as_array().unwrap();
    assert!(
        sessions
            .iter()
            .all(|session| session["session"]["user_id"].as_i64() == Some(user_id))
    );
    assert!(
        sessions
            .iter()
            .any(|session| session["session"]["id"].as_str() == Some(session_id.as_str()))
    );
}
