use crate::AppState;
use crate::core::lib::user::update_user::{UpdateUser, update_user_by_email};
use axum::extract::State;
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use tracing::error;

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    response_message: String,
    response: Option<()>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    user_email: String,
}

pub async fn logout_user(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    // Remove auth cookie
    let mut cookie = Cookie::new("auth_cookie", "");
    cookie.set_path("/");
    cookie.set_max_age(tower_cookies::cookie::time::Duration::ZERO);
    cookies.remove(cookie);

    let user = update_user_by_email(
        &state.db,
        &payload.user_email,
        UpdateUser {
            access_token: Some("".to_string()),
            refresh_token: Some("".to_string()),
            is_logged_out: Some(true),
        },
    )
    .await;

    match user {
        Ok(rows_affected) if rows_affected > 0 => (
            StatusCode::OK,
            Json(LogoutResponse {
                response_message: "Logout successful".to_string(),
                error: None,
                response: None,
            }),
        ),
        Ok(_) => {
            error!("USER LOGOUT WAS UNSUCCESSFUL!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some("User not found".to_string()),
                    response: None,
                }),
            )
        }
        Err(e) => {
            error!("USER LOGOUT WAS UNSUCCESSFUL!");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutResponse {
                    response_message: "Logout failed".to_string(),
                    error: Some(e.to_string()),
                    response: None,
                }),
            )
        }
    }
}
