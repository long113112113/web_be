use crate::constant::auth::REFRESH_TOKEN_DURATION_DAYS;
use crate::{
    error::AuthError,
    models::user::UserModel,
    repository::{token_repository, user_repository},
    utils::jwt::{TokenType, create_jwt, create_refresh_token, decode_jwt_with_type},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::HashingError(e.to_string()))
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<(String, String, UserModel), AuthError> {
    // Validation is now handled by DTO validators in the handler layer
    match user_repository::find_user_by_email(pool, email).await {
        Ok(Some(_)) => return Err(AuthError::EmailAlreadyExists),
        Ok(None) => {}
        Err(e) => return Err(AuthError::from(e)),
    }
    let hashed_password = hash_password(password)?;
    let user = user_repository::create_user(pool, email, &hashed_password)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key")
                || e.to_string().contains("unique constraint")
            {
                AuthError::EmailAlreadyExists
            } else {
                AuthError::from(e)
            }
        })?;

    let token = create_jwt(&user.id.to_string(), jwt_secret)?;
    let refresh_token = create_refresh_token(&user.id.to_string(), jwt_secret)?;

    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_DURATION_DAYS);
    let token_hash = hash_token(&refresh_token);

    token_repository::create_token(pool, user.id, &token_hash, expires_at).await?;

    Ok((token, refresh_token, user))
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), AuthError> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| AuthError::HashingError(e.to_string()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::InvalidCredentials)
}

pub async fn login_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<(String, String, UserModel), AuthError> {
    let user = user_repository::find_user_by_email(pool, email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    verify_password(password, &user.password_hash)?;

    let token = create_jwt(&user.id.to_string(), jwt_secret)?;
    let refresh_token = create_refresh_token(&user.id.to_string(), jwt_secret)?;

    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_DURATION_DAYS);
    let token_hash = hash_token(&refresh_token);

    token_repository::create_token(pool, user.id, &token_hash, expires_at).await?;

    Ok((token, refresh_token, user))
}

pub async fn refresh_access_token(
    pool: &PgPool,
    refresh_token: &str,
    jwt_secret: &str,
) -> Result<(String, String, UserModel), AuthError> {
    let claims = decode_jwt_with_type(refresh_token, jwt_secret, TokenType::Refresh)
        .map_err(|_| AuthError::InvalidCredentials)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenCreationError("Invalid user ID".to_string()))?;

    let token_hash = hash_token(refresh_token);

    let token_record = token_repository::find_token_by_hash(pool, &token_hash)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if token_record.used || token_record.user_id != user_id {
        return Err(AuthError::InvalidCredentials);
    }

    if token_record.expires_at < Utc::now() {
        return Err(AuthError::InvalidCredentials);
    }

    token_repository::set_token_used(pool, &token_hash).await?;

    let user = user_repository::find_user_by_id(pool, user_id)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    let new_access_token = create_jwt(&user.id.to_string(), jwt_secret)?;
    let new_refresh_token = create_refresh_token(&user.id.to_string(), jwt_secret)?;

    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_DURATION_DAYS);
    let new_token_hash = hash_token(&new_refresh_token);

    token_repository::create_token(pool, user.id, &new_token_hash, expires_at).await?;

    Ok((new_access_token, new_refresh_token, user))
}

/// Invalidate a refresh token by marking it as used in the database.
/// This should be called during logout to prevent token reuse.
pub async fn invalidate_refresh_token(pool: &PgPool, refresh_token: &str) -> Result<(), AuthError> {
    let token_hash = hash_token(refresh_token);
    token_repository::set_token_used(pool, &token_hash).await?;
    Ok(())
}
