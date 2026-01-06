use crate::error::AuthError;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_jwt(user_id: &str, secret: &str) -> Result<String, AuthError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .ok_or_else(|| AuthError::TokenCreationError("Invalid timestamp calculation".to_string()))?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        iat: Utc::now().timestamp() as usize,
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::TokenCreationError(e.to_string()))
}

pub fn create_refresh_token(user_id: &str, secret: &str) -> Result<String, AuthError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(
            crate::constant::auth::REFRESH_TOKEN_DURATION_DAYS,
        ))
        .ok_or_else(|| AuthError::TokenCreationError("Invalid timestamp calculation".to_string()))?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        iat: Utc::now().timestamp() as usize,
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::TokenCreationError(e.to_string()))
}
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, AuthError> {
    jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AuthError::TokenCreationError(e.to_string())) // Reusing TokenCreationError for decoding error for now
}
