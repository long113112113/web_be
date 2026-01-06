use crate::handlers::auth::{login_handler, register_handler};
use axum::{Router, routing::post};
use sqlx::PgPool;

pub fn auth_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .with_state(pool)
}
