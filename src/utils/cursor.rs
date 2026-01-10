use crate::error::AppError;
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;

pub fn encode_cursor(full_name: &str, user_id: Uuid) -> String {
    let cursor_data = format!("{}|{}", full_name, user_id);
    general_purpose::STANDARD.encode(cursor_data.as_bytes())
}

pub fn decode_cursor(cursor: &str) -> Result<(String, Uuid), AppError> {
    let decoded = general_purpose::STANDARD
        .decode(cursor)
        .map_err(|_| AppError::BadRequest("Invalid cursor format".into()))?;

    let cursor_str = String::from_utf8(decoded)
        .map_err(|_| AppError::BadRequest("Invalid cursor encoding".into()))?;

    let parts: Vec<&str> = cursor_str.split('|').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest("Invalid cursor structure".into()));
    }

    let full_name = parts[0].to_string();
    let user_id = Uuid::parse_str(parts[1])
        .map_err(|_| AppError::BadRequest("Invalid user ID in cursor".into()))?;

    Ok((full_name, user_id))
}
