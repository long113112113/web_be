use crate::models::token::RefreshToken;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken, sqlx::Error> {
    sqlx::query_as!(
        RefreshToken,
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        user_id,
        token_hash,
        expires_at
    )
    .fetch_one(pool)
    .await
}

pub async fn find_token_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<RefreshToken>, sqlx::Error> {
    sqlx::query_as!(
        RefreshToken,
        r#"
        SELECT * FROM refresh_tokens
        WHERE token_hash = $1
        "#,
        token_hash
    )
    .fetch_optional(pool)
    .await
}

pub async fn set_token_used(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET used = TRUE
        WHERE token_hash = $1
        "#,
        token_hash
    )
    .execute(pool)
    .await?;
    Ok(())
}
