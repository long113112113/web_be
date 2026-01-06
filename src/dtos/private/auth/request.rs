use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}
