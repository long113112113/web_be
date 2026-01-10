use crate::middlewares::auth::auth_middleware;
use crate::state::AppState;
use axum::{Router, middleware::from_fn_with_state};

mod friend_routes;
mod user_routes;

pub fn private_routes(state: AppState) -> Router {
    Router::new()
        // Nest all private route modules here
        .nest("/user", user_routes::user_routes(state.clone()))
        .nest("/friends", friend_routes::friend_routes(state.clone()))
        // Apply auth middleware to all private routes
        .route_layer(from_fn_with_state(state, auth_middleware))
}
