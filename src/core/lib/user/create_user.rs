use crate::core::structs::user::RegisteredUserProfile;
use sqlx::PgPool;

pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub profile_image: String,
    pub country: String,
    pub phone_number: String,
}

pub async fn create_user(
    db: &PgPool,
    user: CreateUser,
) -> Result<RegisteredUserProfile, sqlx::Error> {
    sqlx::query_as::<_, RegisteredUserProfile>(
        r#"
        INSERT INTO users (
            email,
            password,
            full_name,
            profile_image,
            country,
            phone_number
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            full_name,
            email,
            profile_image,
            country,
            phone_number,
            created_at,
            updated_at
        "#,
    )
    .bind(user.email)
    .bind(user.password)
    .bind(user.full_name)
    .bind(user.profile_image)
    .bind(user.country)
    .bind(user.phone_number)
    .fetch_one(db)
    .await
}
