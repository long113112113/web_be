use image::{ImageFormat, ImageReader};
use std::io::Cursor;

use crate::error::AppError;

/// Strips EXIF/metadata from an image by re-encoding it.
/// Returns the cleaned image bytes.
pub fn strip_metadata(data: &[u8], content_type: &str) -> Result<Vec<u8>, AppError> {
    // Determine output format
    let format = match content_type {
        "image/jpeg" | "image/jpg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/gif" => ImageFormat::Gif,
        "image/webp" => ImageFormat::WebP,
        _ => {
            return Err(AppError::InternalError(
                "Unsupported image format".to_string(),
            ));
        }
    };

    // Decode image (this discards metadata)
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| AppError::InternalError(format!("Failed to read image: {}", e)))?
        .decode()
        .map_err(|e| AppError::InternalError(format!("Failed to decode image: {}", e)))?;

    // Re-encode to strip metadata
    let mut output = Cursor::new(Vec::new());
    img.write_to(&mut output, format)
        .map_err(|e| AppError::InternalError(format!("Failed to encode image: {}", e)))?;

    Ok(output.into_inner())
}
