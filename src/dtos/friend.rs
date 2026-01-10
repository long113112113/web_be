use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct FriendResponseDto {
    pub user_id: Uuid,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: String, // 'friend', 'pending_sent', 'pending_received'
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GetFriendsQuery {
    pub cursor: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedFriendsResponse {
    pub data: Vec<FriendResponseDto>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}
