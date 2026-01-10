pub const MAX_AVATAR_SIZE: usize = 5 * 1024 * 1024; // 5MB
pub const ALLOWED_CONTENT_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/gif", "image/webp"];
pub const MAX_PIXELS: u32 = 25_000_000; // 25 megapixels (~5000x5000), prevents memory bomb attacks
