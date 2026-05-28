use crate::core::structs::user::{UserLookUp, UserProfile};
use sqlx::PgPool;

pub async fn find_user_profile_by_email(
    db: &PgPool,
    email: &str,
) -> Result<Option<UserProfile>, sqlx::Error> {
    sqlx::query_as::<_, UserProfile>(
        "SELECT id, full_name, email, profile_image, password, is_active, is_admin, country, phone_number, is_logged_out, status, created_at, updated_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(db)
    .await
}

pub async fn find_user_by_email(
    db: &PgPool,
    email: &str,
) -> Result<Option<UserLookUp>, sqlx::Error> {
    sqlx::query_as::<_, UserLookUp>(
        r#"
        SELECT
            email,
            phone_number,
            created_at,
            updated_at
        FROM users
        WHERE email = $1
        LIMIT 1
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await
}

pub async fn find_user_by_phone_number(
    db: &PgPool,
    phone_number: &str,
) -> Result<Option<UserLookUp>, sqlx::Error> {
    sqlx::query_as::<_, UserLookUp>(
        r#"
        SELECT
            email,
            phone_number,
            created_at,
            updated_at
        FROM users
        WHERE phone_number = $1
        LIMIT 1
        "#,
    )
    .bind(phone_number)
    .fetch_optional(db)
    .await
}
