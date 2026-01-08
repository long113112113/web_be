use crate::handlers::profile::{me_handler, upload_avatar_handler};
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn user_routes(state: AppState) -> Router {
    Router::new()
        .route("/me", get(me_handler))
        .route("/avatar", post(upload_avatar_handler))
        // Add more user-related protected routes here
        // .route("/profile", get(profile_handler))
        // .route("/settings", put(update_settings_handler))
        .with_state(state)
}
