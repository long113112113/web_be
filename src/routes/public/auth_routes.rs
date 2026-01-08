use crate::handlers::auth::{
    login_handler, logout_handler, refresh_token_handler, register_handler,
};
use crate::state::AppState;
use axum::{Router, routing::post};

pub fn auth_routes(state: AppState) -> Router {
    // Routes with rate limiting for brute force protection
    // Uses shared config from AppState (per docs: do not create config multiple times!)
    let rate_limited = Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .layer(tower_governor::GovernorLayer::new(
            state.rate_limit_config.clone(),
        ));

    // Routes without rate limiting
    let non_limited = Router::new()
        .route("/refresh-token", post(refresh_token_handler))
        .route("/logout", post(logout_handler));

    Router::new()
        .merge(rate_limited)
        .merge(non_limited)
        .with_state(state)
}
