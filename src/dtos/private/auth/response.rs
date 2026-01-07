use crate::models::user::UserModel;
use serde::Serialize;

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: UserModel,
}
