use crate::core::structs::sub_session::SubSession;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CreateSubSession {
    pub session_id: Uuid,
    pub user_id: i64,
    pub activity_type: String,
    pub activity_description: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: String,
    pub request_path: String,
}

pub async fn create_sub_session(
    db: &PgPool,
    sub_session: CreateSubSession,
) -> Result<SubSession, sqlx::Error> {
    sqlx::query_as::<_, SubSession>(
        r#"
        INSERT INTO sub_sessions (
            id,
            session_id,
            user_id,
            activity_type,
            activity_description,
            ip_address,
            user_agent,
            request_method,
            request_path
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
            id,
            creation_order,
            session_id,
            user_id,
            activity_type,
            activity_description,
            ip_address,
            user_agent,
            request_method,
            request_path,
            created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sub_session.session_id)
    .bind(sub_session.user_id)
    .bind(sub_session.activity_type)
    .bind(sub_session.activity_description)
    .bind(sub_session.ip_address)
    .bind(sub_session.user_agent)
    .bind(sub_session.request_method)
    .bind(sub_session.request_path)
    .fetch_one(db)
    .await
}
