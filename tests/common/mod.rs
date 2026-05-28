use auth::db::connect_postgres::connect_pg;
use auth::utils::load_config::load_config;
use auth::{AppState, create_app};
use axum_test::TestServer;
use std::sync::Arc;

pub async fn setup_test_server() -> TestServer {
    dotenvy::from_filename(".env.development").ok();

    let app_config = load_config().expect("Failed to load config");

    let db_config = app_config
        .database
        .as_ref()
        .expect("SERVER START-UP ERROR: DATABASE CONFIGURATION IS MISSING!");

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_config
            .user
            .as_deref()
            .expect("SERVER START-UP ERROR: DATABASE USER IS MISSING!"),
        db_config
            .password
            .as_deref()
            .expect("SERVER START-UP ERROR: DATABASE PASSWORD IS MISSING!"),
        db_config.host,
        db_config.port,
        db_config.name
    );

    let db_pool = connect_pg(
        database_url,
        db_config.max_connections,
        db_config.connect_timeout_secs,
    )
    .await;

    let state = AppState {
        config: Arc::new(app_config),
        db: db_pool,
    };

    let app = create_app(state);
    TestServer::new(app).expect("Failed to create test server")
}

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Serialize)]
pub struct RegisterRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub country: String,
    pub phone_number: String,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestRegisterResponse {
    pub response_message: String,
    pub response: Option<TestResponseCore>,
    pub error: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestLoginResponse {
    pub response_message: String,
    pub response: Option<TestResponseCore>,
    pub error: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestResponseCore {
    pub user_profile: Option<TestUserProfile>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TestUserProfile {
    pub id: i64,
    pub full_name: String,
    pub email: String,
}
