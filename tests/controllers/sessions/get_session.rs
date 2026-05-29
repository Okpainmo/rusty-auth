use crate::common::{RegisterRequest, TestRegisterResponse, setup_test_server};
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_get_session_success() {
    let server = setup_test_server().await;

    let unique_id = Uuid::new_v4().to_string();
    let register_response = server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Session".to_string(),
            last_name: "User".to_string(),
            email: format!("session_get_{}@example.com", unique_id),
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
        .get(&format!("/api/v1/auth/sessions/{}", session_id))
        .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<Value>();
    assert_eq!(
        body["response"]["session"]["id"].as_str().unwrap(),
        session_id
    );
    assert_eq!(
        body["response"]["session"]["sub_sessions"][0]["activity_type"]
            .as_str()
            .unwrap(),
        "register"
    );
}
