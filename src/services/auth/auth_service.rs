use crate::constant::auth::{MIN_PASSWORD_LENGTH, REFRESH_TOKEN_DURATION_DAYS};
use crate::{
    error::AuthError,
    models::user::UserModel,
    repository::{token_repository, user_repository},
    utils::jwt::{create_jwt, create_refresh_token, decode_jwt},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;
use validator::ValidateEmail;


fn is_valid_email(email: &str) -> bool {
    email.validate_email()
}
pub fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AuthError::WeakPassword(format!(
            "Password must be at least {} characters",
            MIN_PASSWORD_LENGTH
        )));
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_digit || !has_special {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one uppercase letter, one lowercase letter, one digit, and one special character".to_string(),
        ));
    }

    Ok(())
}

fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::HashingError(e.to_string()))
}
pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<(String, String, UserModel), AuthError> {
    if !is_valid_email(email) {
        return Err(AuthError::InvalidEmail);
    }
    validate_password(password)?;
    match user_repository::find_user_by_email(pool, email).await {
        Ok(Some(_)) => return Err(AuthError::EmailAlreadyExists),
        Ok(None) => {}
        Err(e) => return Err(AuthError::DatabaseError(e.to_string())),
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
                AuthError::DatabaseError(e.to_string())
            }
        })?;

    let token = create_jwt(&user.id.to_string(), jwt_secret)?;
    let refresh_token = create_refresh_token(&user.id.to_string(), jwt_secret)?;

    // Calculate expiration time for database
    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_DURATION_DAYS);

    // Hash the refresh token
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Save refresh token to database
    token_repository::create_token(pool, user.id, &token_hash, expires_at)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

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
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    verify_password(password, &user.password_hash)?;

    let token = create_jwt(&user.id.to_string(), jwt_secret)?;
    let refresh_token = create_refresh_token(&user.id.to_string(), jwt_secret)?;

    // Calculate expiration time for database
    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_DURATION_DAYS);

    // Hash the refresh token
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Save refresh token to database
    token_repository::create_token(pool, user.id, &token_hash, expires_at)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok((token, refresh_token, user))
}
// Refresh access token
pub async fn refresh_access_token(
    pool: &PgPool,
    refresh_token: &str,
    jwt_secret: &str,
) -> Result<(String, String, UserModel), AuthError> {
    // Decode and validate token signature/expiration
    let claims =
        decode_jwt(refresh_token, jwt_secret).map_err(|_| AuthError::InvalidCredentials)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenCreationError("Invalid user ID".to_string()))?;

    // Hash the refresh token to look it up
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Find token in DB
    let token_record = token_repository::find_token_by_hash(pool, &token_hash)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    // Check if used or mismatched user
    if token_record.used || token_record.user_id != user_id {
        return Err(AuthError::InvalidCredentials);
    }

    // Check expiration (DB check)
    if token_record.expires_at < Utc::now() {
        return Err(AuthError::InvalidCredentials);
    }

    // Mark old token as used
    token_repository::set_token_used(pool, &token_hash)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Find user
    let user = user_repository::find_user_by_id(pool, user_id)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    // Generate NEW tokens
    let new_access_token = create_jwt(&user.id.to_string(), jwt_secret)?;
    let new_refresh_token = create_refresh_token(&user.id.to_string(), jwt_secret)?;

    // Calculate expiration time for new refresh token
    let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_DURATION_DAYS);

    // Hash the new refresh token
    let mut hasher_new = Sha256::new();
    hasher_new.update(new_refresh_token.as_bytes());
    let new_token_hash = format!("{:x}", hasher_new.finalize());

    // Save new refresh token to database
    token_repository::create_token(pool, user.id, &new_token_hash, expires_at)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok((new_access_token, new_refresh_token, user))
}
