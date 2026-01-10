use crate::models::profile::ProfileModel;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ProfileModel>, sqlx::Error> {
    sqlx::query_as::<_, ProfileModel>(
        "SELECT id, user_id, full_name, bio, avatar_url, created_at, updated_at FROM profiles WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_profile(pool: &PgPool, user_id: Uuid) -> Result<ProfileModel, sqlx::Error> {
    sqlx::query_as::<_, ProfileModel>(
        "INSERT INTO profiles (user_id) VALUES ($1) RETURNING id, user_id, full_name, bio, avatar_url, created_at, updated_at"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn update_avatar_url(
    pool: &PgPool,
    user_id: Uuid,
    avatar_url: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE profiles SET avatar_url = $1 WHERE user_id = $2")
        .bind(avatar_url)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Ensures a profile exists for the user, creating one if needed
/// Uses INSERT ON CONFLICT for atomic operation (prevents race conditions)
pub async fn ensure_profile_exists(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<ProfileModel, sqlx::Error> {
    sqlx::query_as::<_, ProfileModel>(
        r#"
        INSERT INTO profiles (user_id) 
        VALUES ($1)
        ON CONFLICT (user_id) DO UPDATE 
        SET user_id = EXCLUDED.user_id
        RETURNING id, user_id, full_name, bio, avatar_url, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Updates profile fields dynamically based on what's provided
/// Uses QueryBuilder for efficient and maintainable query construction
pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    full_name: Option<&str>,
    bio: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<ProfileModel, sqlx::Error> {
    let mut builder = sqlx::QueryBuilder::new("UPDATE profiles SET ");
    let mut separated = builder.separated(", ");

    if let Some(name) = full_name {
        separated.push("full_name = ");
        separated.push_bind_unseparated(name);
    }
    if let Some(b) = bio {
        separated.push("bio = ");
        separated.push_bind_unseparated(b);
    }
    if let Some(url) = avatar_url {
        separated.push("avatar_url = ");
        separated.push_bind_unseparated(url);
    }

    builder.push(" WHERE user_id = ");
    builder.push_bind(user_id);
    builder.push(" RETURNING id, user_id, full_name, bio, avatar_url, created_at, updated_at");

    builder
        .build_query_as::<ProfileModel>()
        .fetch_one(pool)
        .await
}
