use crate::{
    config::Config,
    dtos::private::auth::{
        request::{LoginRequest, RefreshTokenRequest, RegisterRequest},
        response::{LoginResponse, RegisterResponse},
    },
    services::auth::auth_service,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;

pub async fn register_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let config = Config::init();
    match auth_service::register_user(&pool, &payload.email, &payload.password, &config.jwt_secret)
        .await
    {
        Ok((token, refresh_token, user)) => (
            StatusCode::CREATED,
            Json(RegisterResponse {
                token,
                refresh_token,
                user,
            }),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn login_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // In a real production app, cache config in State
    let config = Config::init();

    match auth_service::login_user(&pool, &payload.email, &payload.password, &config.jwt_secret)
        .await
    {
        Ok((token, refresh_token, user)) => (
            StatusCode::OK,
            Json(LoginResponse {
                token,
                refresh_token,
                user,
            }),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn refresh_token_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<RefreshTokenRequest>,
) -> impl IntoResponse {
    let config = Config::init();

    match auth_service::refresh_access_token(&pool, &payload.refresh_token, &config.jwt_secret)
        .await
    {
        Ok((token, refresh_token, user)) => (
            StatusCode::OK,
            Json(LoginResponse {
                token,
                refresh_token,
                user,
            }),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}
