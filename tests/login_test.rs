mod common;

use common::{LoginRequest, RegisterRequest, TestLoginResponse, setup_test_server};
use uuid::Uuid;

#[tokio::test]
async fn test_login_user_success() {
    let server = setup_test_server().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("login_{}@example.com", unique_id);
    let password = "secure_password123";

    // Register first
    server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Login".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: password.to_string(),
            country: "TestCountry".to_string(),
            phone_number: unique_id[0..10].to_string(),
        })
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Login
    let response = server
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: email.clone(),
            password: password.to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::OK);
    let body = response.json::<TestLoginResponse>();
    assert_eq!(body.response_message, "Login successful");
    let res = body.response.unwrap();
    assert!(res.access_token.is_some());
    assert!(res.refresh_token.is_some());

    // Check if cookie is set
    let _ = response.cookie("auth_cookie");
}

#[tokio::test]
async fn test_login_user_invalid_password() {
    let server = setup_test_server().await;

    let unique_id = Uuid::new_v4().to_string();
    let email = format!("wrong_pass_{}@example.com", unique_id);
    let password = "correct_password";

    // Register first
    server
        .post("/api/v1/auth/register")
        .json(&RegisterRequest {
            first_name: "Login".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password: password.to_string(),
            country: "TestCountry".to_string(),
            phone_number: unique_id[0..10].to_string(),
        })
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Login with wrong password
    let response = server
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: email.clone(),
            password: "wrong_password".to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    let body = response.json::<TestLoginResponse>();
    assert_eq!(body.error.unwrap(), "Invalid email or password");
}

#[tokio::test]
async fn test_login_non_existent_user() {
    let server = setup_test_server().await;

    let response = server
        .post("/api/v1/auth/login")
        .json(&LoginRequest {
            email: "non_existent@example.com".to_string(),
            password: "any_password".to_string(),
        })
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}
