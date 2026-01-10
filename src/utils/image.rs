use image::{ImageFormat, ImageReader};
use std::io::Cursor;

use crate::constant::image::MAX_PIXELS;
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
            return Err(AppError::InternalError("Unsupported image format".into()));
        }
    };

    // SECURITY: Check image dimensions BEFORE decoding to prevent memory bomb attacks
    // This reads only the header, not the entire image data
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| AppError::InternalError(format!("Failed to read image: {}", e).into()))?;

    let (width, height) = reader.into_dimensions().map_err(|e| {
        AppError::InternalError(format!("Failed to get image dimensions: {}", e).into())
    })?;

    if width as u64 * height as u64 > MAX_PIXELS as u64 {
        return Err(AppError::BadRequest(
            format!(
                "Image dimensions too large: {}x{} pixels. Maximum allowed: {} total pixels",
                width, height, MAX_PIXELS
            )
            .into(),
        ));
    }

    // Now it's safe to decode the image (dimensions are validated)
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| AppError::InternalError(format!("Failed to read image: {}", e).into()))?
        .decode()
        .map_err(|e| AppError::InternalError(format!("Failed to decode image: {}", e).into()))?;

    // Re-encode to strip metadata
    let mut output = Cursor::new(Vec::new());
    img.write_to(&mut output, format)
        .map_err(|e| AppError::InternalError(format!("Failed to encode image: {}", e).into()))?;

    Ok(output.into_inner())
}
