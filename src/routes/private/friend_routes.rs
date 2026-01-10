use crate::handlers::friend::{
    accept_request_handler, delete_friend_handler, get_friends_handler,
    get_pending_requests_handler, get_sent_requests_handler, send_request_handler,
};
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn friend_routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_friends_handler))
        .route("/pending", get(get_pending_requests_handler))
        .route("/sent", get(get_sent_requests_handler))
        .route("/request/{target_id}", post(send_request_handler))
        .route("/accept/{target_id}", post(accept_request_handler))
        .route("/{target_id}", delete(delete_friend_handler))
        .with_state(state)
}
