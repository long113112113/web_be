# Performance Analysis Report

## Executive Summary
- **Overall performance grade**: 7.5/10
- **Critical issues**: 1
- **Optimization opportunities**: 4

The application is well-structured with a solid foundation using Axum and Tokio. The use of `spawn_blocking` for CPU-intensive tasks is correct. However, there are significant performance risks related to memory usage during image processing, potential N+1 query issues in database interactions, and ambiguous rate limit configuration.

## Findings by Category

### 1. Async Runtime
**Grade**: 9/10
- **Issues found**: None critical.
- **Good practices**: `tokio::task::spawn_blocking` is correctly used in `src/handlers/profile.rs` to offload image processing.
- **Recommendations**: Continue monitoring blocking tasks. Ensure no other CPU-heavy operations creep into async handlers.

### 2. Database
**Grade**: 7/10
- **Issues**:
    - **N+1 Queries**: `me_handler` performs multiple sequential queries (`find_user_by_id`, then `ensure_profile_exists` which does `find` or `create`). This increases latency.
    - **Race Condition**: `ensure_profile_exists` (SELECT then INSERT) is prone to race conditions under high concurrency.
    - **Connection Pool**: `max_connections(10)` is relatively low for high-throughput scenarios, though fine for start.
    - **Dynamic Query Building**: `update_profile` builds query strings manually. While functional, it's less efficient than prepared statements and harder to maintain.
- **Optimizations**:
    - Use `JOIN` to fetch user and profile in a single query.
    - Use `INSERT ... ON CONFLICT DO NOTHING` (or `DO UPDATE`) to handle profile existence atomically.
    - Increase `max_connections` if database metrics show contention.

### 3. Memory Allocation & Usage
**Grade**: 6/10
- **Issues**:
    - **Image Processing Overhead**: `strip_metadata` decodes the entire image into a bitmap (`DynamicImage`), which consumes `width * height * 4` bytes of RAM. For a 5MB compressed image, this could be 50MB+ in memory. It then re-encodes it.
    - **Buffer Copying**: `process_and_upload_avatar` takes `Vec<u8>`, passes it to `strip_metadata` (which creates a *new* `Vec<u8>`), and then uploads the new one. The original vector remains until scope end.
    - **Multipart Buffering**: The handlers read the entire file into memory (`field.bytes().await.to_vec()`). For 5MB files, this is acceptable, but limits scalability.
    - **Excessive String Allocations**: Widespread use of `.to_string()` for error messages (e.g., `AppError::BadRequest("...".to_string())`). This creates unnecessary heap allocations on every error.
- **Optimizations**:
    - **Critical**: Use streaming or limits on image dimensions (not just file size) to prevent "zip bomb" attacks (small file, huge resolution).
    - Use `Cow<'static, str>` for error messages to avoid allocation.

### 4. HTTP Request/Response Performance
**Grade**: 8/10
- **Issues**:
    - **Rate Limit Ambiguity**: `tower_governor` is configured with `per_second(60)`. The comment says "replenishes one every 60 seconds", but `per_second(60)` usually means 60 requests per second. If it is 60 req/s with burst 5, the burst is negligible. If it is 1 req/min, the code is likely wrong.
- **Optimizations**:
    - Clarify and fix rate limit configuration.
    - Verify middleware chain overhead.

### 5. External Service Calls
**Grade**: 8/10
- **Issues**:
    - **S3 Uploads**: Files are fully buffered in memory before upload. No streaming upload is implemented.
- **Optimizations**:
    - Implementing streaming uploads to S3 (intercepting the stream) would significantly reduce memory footprint, although it makes "stripping metadata" harder (requires streaming processor).

---

## Priority Recommendations

### P0 (Critical - Immediate fix)
1.  **Fix Rate Limit Configuration**: Clarify if `per_second(60)` (60 req/s) or 1 req/min is intended. Currently, there is a contradiction between code and comments, potentially leaving the API wide open or too restricted.
    *   **Impact**: Security / Usability.
2.  **Image Dimension Check**: Before fully decoding an image in `strip_metadata`, read only the header to check dimensions. Reject images that are too large (pixel-wise) to prevent DoS/OOM.
    *   **Impact**: Stability.

### P1 (High - Fix soon)
1.  **Optimize `me_handler` Queries**: Refactor to use a single SQL query with `JOIN` to fetch user and profile data.
    *   **Impact**: ~30-50% latency reduction for this endpoint.
2.  **Fix Race Condition in `ensure_profile_exists`**: Use `INSERT ... ON CONFLICT` SQL statement.
    *   **Impact**: Data integrity and stability under load.
3.  **Reduce Error String Allocations**: Refactor `AppError` to use `Cow<'static, str>` or `&'static str`.
    *   **Impact**: Reduced allocator pressure.

### P2 (Medium - Consider)
1.  **Streaming Uploads**: Implement streaming for file uploads to S3 to reduce memory usage.
    *   **Impact**: Lower memory footprint, better scalability.
2.  **Dynamic Query Builder**: Switch to `sqlx`'s `QueryBuilder` for `update_profile` or cleaner SQL construction.

---

## Estimated Performance Gains

- **Database Optimization**: reducing 3 queries to 1 in `me_handler` -> **~5-10ms latency reduction** per call (assuming local DB, more for remote).
- **Memory Optimization**: checking image dimensions -> Prevents **OOM crashes** (infinite gain in stability).
- **String Allocation Removal**: Minor throughput gain, but significant reduction in memory churn (GC-like behavior in allocator).

## Code Examples

### Database Optimization (P1)
**Current (`src/repository/profile_repository.rs`):**
```rust
pub async fn ensure_profile_exists(pool: &PgPool, user_id: Uuid) -> Result<ProfileModel, sqlx::Error> {
    match find_by_user_id(pool, user_id).await? {
        Some(profile) => Ok(profile),
        None => create_profile(pool, user_id).await,
    }
}
```
**Optimized:**
```rust
pub async fn ensure_profile_exists(pool: &PgPool, user_id: Uuid) -> Result<ProfileModel, sqlx::Error> {
    sqlx::query_as::<_, ProfileModel>(
        r#"
        INSERT INTO profiles (user_id) VALUES ($1)
        ON CONFLICT (user_id) DO UPDATE SET user_id = EXCLUDED.user_id -- Dummy update to return row
        RETURNING id, user_id, full_name, bio, avatar_url, created_at, updated_at
        "#
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}
```

### Image Security (P0)
**Current (`src/utils/image.rs`):**
```rust
let img = ImageReader::new(Cursor::new(data))
    .with_guessed_format()?
    .decode()?; // ❌ Decodes blindly
```
**Optimized:**
```rust
let reader = ImageReader::new(Cursor::new(data))
    .with_guessed_format()?;
let (width, height) = reader.into_dimensions()?; // ✅ Check dimensions first
if width * height > MAX_PIXELS {
    return Err(AppError::BadRequest("Image too large".to_string()));
}
// Re-create reader or seek to start to decode
let img = ImageReader::new(Cursor::new(data))
    .with_guessed_format()?
    .decode()?;
```
