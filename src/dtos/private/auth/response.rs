use crate::models::user::UserModel;
use serde::Serialize;

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserModel,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub email: String,
}
