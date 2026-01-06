use crate::{
    config::Config,
    dtos::{
        private::auth::request::{LoginRequest, RegisterRequest},
        private::auth::response::{LoginResponse, RegisterResponse},
    },
    services::auth::auth_service,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;

pub async fn register_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match auth_service::register_user(&pool, &payload.email, &payload.password).await {
        Ok(user) => (
            StatusCode::CREATED,
            Json(RegisterResponse { email: user.email }),
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
        Ok((token, user)) => (StatusCode::OK, Json(LoginResponse { token, user })).into_response(),
        Err(e) => e.into_response(),
    }
}
