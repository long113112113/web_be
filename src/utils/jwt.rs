use crate::error::AuthError;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Access,
    Refresh,
}

impl TokenType {
    pub fn as_str(&self) -> &str {
        match self {
            TokenType::Access => "access",
            TokenType::Refresh => "refresh",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    pub token_type: String,
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
        token_type: TokenType::Access.as_str().to_string(),
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
        token_type: TokenType::Refresh.as_str().to_string(),
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

pub fn decode_jwt_with_type(
    token: &str,
    secret: &str,
    expected_type: TokenType,
) -> Result<Claims, AuthError> {
    let claims = decode_jwt(token, secret)?;

    if claims.token_type != expected_type.as_str() {
        return Err(AuthError::InvalidTokenType);
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AuthError;
    use chrono::Utc;

    const SECRET: &str = "test_secret";
    const USER_ID: &str = "user_123";

    #[test]
    fn test_create_jwt_happy_path() {
        let token_result = create_jwt(USER_ID, SECRET);
        assert!(token_result.is_ok());
        let token = token_result.unwrap();
        assert!(!token.is_empty());

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_decode_jwt_happy_path() {
        let token = create_jwt(USER_ID, SECRET).expect("Failed to create token");
        let claims_result = decode_jwt(&token, SECRET);

        assert!(claims_result.is_ok());
        let claims = claims_result.unwrap();
        assert_eq!(claims.sub, USER_ID);

        // Verify expiration is in the future
        let now = Utc::now().timestamp() as usize;
        assert!(claims.exp > now);
    }

    #[test]
    fn test_create_refresh_token_happy_path() {
        let token_result = create_refresh_token(USER_ID, SECRET);
        assert!(token_result.is_ok());
        let token = token_result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_decode_jwt_invalid_token() {
        let invalid_token = "invalid.token.string";
        let result = decode_jwt(invalid_token, SECRET);
        assert!(matches!(result, Err(AuthError::TokenCreationError(_))));
    }

    #[test]
    fn test_decode_jwt_wrong_secret() {
        let token = create_jwt(USER_ID, SECRET).expect("Failed to create token");
        let wrong_secret = "wrong_secret";
        let result = decode_jwt(&token, wrong_secret);
        assert!(matches!(result, Err(AuthError::TokenCreationError(_))));
    }

    #[test]
    fn test_access_token_has_correct_type() {
        let token = create_jwt(USER_ID, SECRET).expect("Failed to create token");
        let claims = decode_jwt(&token, SECRET).expect("Failed to decode token");
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_refresh_token_has_correct_type() {
        let token = create_refresh_token(USER_ID, SECRET).expect("Failed to create token");
        let claims = decode_jwt(&token, SECRET).expect("Failed to decode token");
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_decode_access_token_with_correct_type() {
        let token = create_jwt(USER_ID, SECRET).expect("Failed to create token");
        let result = decode_jwt_with_type(&token, SECRET, TokenType::Access);
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, USER_ID);
    }

    #[test]
    fn test_decode_refresh_token_with_correct_type() {
        let token = create_refresh_token(USER_ID, SECRET).expect("Failed to create token");
        let result = decode_jwt_with_type(&token, SECRET, TokenType::Refresh);
        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, USER_ID);
    }

    #[test]
    fn test_token_substitution_access_as_refresh_fails() {
        let access_token = create_jwt(USER_ID, SECRET).expect("Failed to create token");
        let result = decode_jwt_with_type(&access_token, SECRET, TokenType::Refresh);
        assert!(matches!(result, Err(AuthError::InvalidTokenType)));
    }

    #[test]
    fn test_token_substitution_refresh_as_access_fails() {
        let refresh_token = create_refresh_token(USER_ID, SECRET).expect("Failed to create token");
        let result = decode_jwt_with_type(&refresh_token, SECRET, TokenType::Access);
        assert!(matches!(result, Err(AuthError::InvalidTokenType)));
    }
}
