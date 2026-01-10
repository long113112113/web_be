use chrono::{DateTime, Utc};
use serde::Serialize;

use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct FriendResponseDto {
    pub user_id: Uuid,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: String, // 'friend', 'pending_sent', 'pending_received'
    pub created_at: DateTime<Utc>,
}
