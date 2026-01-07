use crate::state::AppState;
use axum::Router;

mod auth_routes;

pub fn public_routes(state: AppState) -> Router {
    Router::new()
        // Nest all public route modules here
        .nest("/auth", auth_routes::auth_routes(state))
    // Add more public routes here
    // .nest("/health", health_routes::health_routes())
}
