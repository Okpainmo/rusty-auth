use crate::AppState;
use crate::core::lib::user::find_user::find_user_profile_by_email;
use crate::core::lib::user::update_user::{UpdateUser, update_user_by_email};
use crate::core::structs::user::UserProfile;
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::generate_tokens::{User, generate_tokens};
use crate::utils::hashing_handler::hashing_handler;
use crate::utils::verification_handler::verification_handler; // your existing password verification function
use axum::extract::State;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct ResponseCore {
    user_profile: UserProfile,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    response_message: String,
    response: Option<ResponseCore>,
    error: Option<String>,
}

// Reuse UserProfile and ResponseCore from register controller

pub async fn login_user(
    cookies: Cookies,
    // Extension(db_pool): Extension<PgPool>,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Fetch user by email
    let user_result = find_user_profile_by_email(&state.db, &payload.email).await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("LOGIN FAILED: PROVIDE EMAIL AND PASSWORD!");

            return (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some("Invalid email or password".to_string()),
                }),
            );
        }
        Err(e) => {
            error!("USER LOGIN FAILED!");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    };

    match verification_handler(&payload.password, &user.password).await {
        Ok(true) => {
            let tokens = match generate_tokens(
                "auth",
                User {
                    id: user.id,
                    email: payload.email.clone(),
                },
                &state.config,
            )
            .await
            {
                Ok(tokens) => tokens,
                Err(e) => {
                    error!("TOKEN GENERATION ERROR!");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginResponse {
                            response_message: "Failed to generate tokens".to_string(),
                            response: None,
                            error: Some(format!("Token generation error: {}", e)),
                        }),
                    );
                }
            };

            let hashed_access_token = match tokens.access_token.as_deref() {
                Some(access_token) => match hashing_handler(access_token).await {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("ACCESS TOKEN HASHING ERROR!");
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(LoginResponse {
                                response_message: "Login failed".to_string(),
                                response: None,
                                error: Some(format!("Access token hashing error: {}", e)),
                            }),
                        );
                    }
                },
                None => None,
            };

            let hashed_refresh_token = match tokens.refresh_token.as_deref() {
                Some(refresh_token) => match hashing_handler(refresh_token).await {
                    Ok(hash) => Some(hash),
                    Err(e) => {
                        error!("REFRESH TOKEN HASHING ERROR!");
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(LoginResponse {
                                response_message: "Login failed".to_string(),
                                response: None,
                                error: Some(format!("Refresh token hashing error: {}", e)),
                            }),
                        );
                    }
                },
                None => None,
            };

            let _ = update_user_by_email(
                &state.db,
                &payload.email,
                UpdateUser {
                    access_token: hashed_access_token,
                    refresh_token: hashed_refresh_token,
                    is_logged_out: Some(false),
                },
            )
            .await;

            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap(), &state.config).await;

            (
                StatusCode::OK,
                Json(LoginResponse {
                    response_message: "Login successful".to_string(),
                    response: Some(ResponseCore {
                        user_profile: UserProfile {
                            is_logged_out: false,
                            ..user
                        },
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                    }),
                    error: None,
                }),
            )
        }
        Ok(false) => {
            error!("USER LOGIN FAILED!");

            (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some("Invalid email or password".to_string()),
                }),
            )
        }
        Err(e) => {
            error!("USER LOGIN FAILED!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    response_message: "Login failed".to_string(),
                    response: None,
                    error: Some(format!("Password verification error: {}", e)),
                }),
            )
        }
    }
}
