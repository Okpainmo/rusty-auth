use crate::AppState;
use crate::core::lib::user::create_user::{CreateUser, create_user};
use crate::core::lib::user::find_user::{find_user_by_email, find_user_by_phone_number};
use crate::core::lib::user::update_user::{UpdateUser, update_user_by_id};
use crate::core::structs::user::RegisteredUserProfile;
use crate::utils::cookie_deploy_handler::deploy_auth_cookie;
use crate::utils::generate_tokens::User;
use crate::utils::generate_tokens::generate_tokens;
use crate::utils::hashing_handler::hashing_handler;
use axum::extract::State;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct InSpecs {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
    country: String,
    phone_number: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseCore {
    user_profile: RegisteredUserProfile,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

// ====== Response Data ======
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    response_message: String,
    response: Option<ResponseCore>,
    error: Option<String>,
}

pub async fn register_user(
    cookies: Cookies,
    State(state): State<AppState>,
    Json(payload): Json<InSpecs>,
) -> impl IntoResponse {
    // Hash the password
    let hashed_password = match hashing_handler(payload.password.as_str()).await {
        Ok(hash) => hash,
        Err(e) => {
            error!("PASSWORD HASHING ERROR!");

            return (
                StatusCode::BAD_REQUEST,
                Json(RegisterResponse {
                    response_message: "Failed to hash password".to_string(),
                    response: None,
                    error: Some(format!("Password hashing error: {}", e)),
                }),
            );
        }
    };

    // ===== Check for existing user by email =====
    let email_query = find_user_by_email(&state.db, &payload.email).await;

    match email_query {
        Ok(Some(_existing_user)) => {
            // Email already exists (query condition)
            error!("REGISTRATION FAILED: EMAIL ALREADY EXISTS");

            return (
                StatusCode::FORBIDDEN,
                Json(RegisterResponse {
                    response_message: "Registration failed".to_string(),
                    response: None,
                    error: Some("Email already exists".to_string()),
                }),
            );
        }

        Ok(None) => {
            // No user with this email exists — continue registration
        }

        Err(e) => {
            error!("DATABASE ERROR WHILE CHECKING USER UNIQUENESS: {}", e);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    response_message: "Registration failed".to_string(),
                    response: None,
                    error: Some(format!("Database error: {}", e)),
                }),
            );
        }
    }

    let phone_number_query = find_user_by_phone_number(&state.db, &payload.phone_number).await;

    match phone_number_query {
        Ok(Some(_existing_user)) => {
            // Email already exists (query condition)
            error!("REGISTRATION FAILED: PHONE NUMBER ALREADY EXISTS");

            return (
                StatusCode::FORBIDDEN,
                Json(RegisterResponse {
                    response_message: "Registration failed".to_string(),
                    response: None,
                    error: Some("Phone number already exists".to_string()),
                }),
            );
        }

        Ok(None) => {
            // No user with this phone_number exists — continue registration
        }

        Err(e) => {
            error!("ERROR WHILE CHECKING USER UNIQUENESS: {}", e);

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    response_message: "Registration failed".to_string(),
                    response: None,
                    error: Some(format!("Server error: {}", e)),
                }),
            );
        }
    }

    let full_name = format!("{} {}", payload.first_name, payload.last_name);

    // Create user
    let result = create_user(
        &state.db,
        CreateUser {
            email: payload.email.clone(),
            password: hashed_password,
            full_name,
            profile_image: "".to_string(),
            country: payload.country,
            phone_number: payload.phone_number,
        },
    )
    .await;

    match result {
        Ok(new_user) => {
            let tokens = match generate_tokens(
                "auth",
                User {
                    id: new_user.id,
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
                        Json(RegisterResponse {
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
                            Json(RegisterResponse {
                                response_message: "Failed to hash access token".to_string(),
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
                            Json(RegisterResponse {
                                response_message: "Failed to hash refresh token".to_string(),
                                response: None,
                                error: Some(format!("Refresh token hashing error: {}", e)),
                            }),
                        );
                    }
                },
                None => None,
            };

            // Update tokens for the created user
            let update_result = update_user_by_id(
                &state.db,
                new_user.id,
                UpdateUser {
                    access_token: hashed_access_token,
                    refresh_token: hashed_refresh_token,
                    is_logged_out: None,
                },
            )
            .await;

            if let Err(e) = update_result {
                error!("FAILED TO UPDATE TOKENS: {}", e);
            }

            deploy_auth_cookie(cookies, tokens.auth_cookie.unwrap(), &state.config).await;

            (
                StatusCode::CREATED,
                Json(RegisterResponse {
                    response_message: format!(
                        "User with email '{}' registered successfully!",
                        &payload.email
                    ),
                    response: Some(ResponseCore {
                        user_profile: new_user,
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                    }),
                    error: None,
                }),
            )
        }
        Err(e) => {
            let error_msg =
                if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                    error!("REGISTRATION FAILED: USER WITH EMAIL ALREADY EXIST!");
                    "Email already exists".to_string()
                } else {
                    error!("REGISTRATION FAILED: AN ERROR OCCURRED WHILE REGISTERING NEW USER!");
                    format!("Database error: {}", e)
                };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    response_message: "Failed to register user".to_string(),
                    response: None,
                    error: Some(error_msg),
                }),
            )
        }
    }
}
