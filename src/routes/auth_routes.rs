use crate::handlers::auth::{login_handler, refresh_token_handler, register_handler};
use crate::state::AppState;
use axum::{Router, routing::post};

pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/refresh-token", post(refresh_token_handler))
        .with_state(state)
}
