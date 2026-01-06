use crate::models::user::UserModel;
use serde::Serialize;

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserModel,
}
