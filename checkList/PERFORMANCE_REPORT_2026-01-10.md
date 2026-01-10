# Performance Analysis Report

## Executive Summary
- **Overall performance grade**: 7/10
- **Critical issues**: 1 (P0)
- **Optimization opportunities**: 4

The application is well-structured with a modern Rust stack (Axum, Tokio, SQLx). However, a critical issue exists in the authentication flow where CPU-intensive password hashing blocks the async runtime, which could severely degrade throughput under load. Database interaction patterns are generally safe but lack transactional integrity in multi-step operations. Memory usage is acceptable for the current scope but naive file handling could become a bottleneck.

## Findings by Category

### 1. Async Runtime
**Grade**: 5/10
- **Issues found**:
  - **CRITICAL**: `Argon2` password hashing (`hash_password`, `verify_password`) in `src/services/auth/auth_service.rs` runs synchronously on the main async thread. This blocks the Tokio worker thread for hundreds of milliseconds per login/register, effectively serializing auth requests and stalling other connections on that thread.
- **Good points**:
  - Image processing (`strip_metadata`) is correctly offloaded to `spawn_blocking` in `src/handlers/profile.rs`.
  - `tokio::main` and scheduler are configured correctly.

### 2. Database
**Grade**: 7/10
- **Issues**:
  - **Lack of Transactions**: `register_user`, `login_user`, and `refresh_access_token` perform multiple database operations (SELECT then INSERT/UPDATE) without a transaction. This risks data inconsistency (e.g., user created but token creation fails) and race conditions.
  - **Pool Configuration**: `max_connections` is set to 10 in `src/main.rs`. While safe for dev, this is likely too low for a production backend serving concurrent traffic.
- **Optimizations**:
  - Wrap multi-step logic in `pool.begin().await` transactions.
  - Tune pool size based on available CPU/Memory and database limits (e.g., 50-100 for production).

### 3. Memory Allocation & Usage
**Grade**: 8/10
- **Issues**:
  - **File Upload Buffering**: `upload_avatar_handler` reads the entire file content into a `Vec<u8>` (`field.bytes().await.to_vec()`) before processing. For 5MB limit, concurrent uploads will cause memory spikes.
  - **String Cloning**: Minor excessive cloning of `content_type` strings in upload handlers.
- **Good points**:
  - Most DTOs and internal logic use appropriate types.

### 4. HTTP Request/Response
**Grade**: 9/10
- **Good points**:
  - Axum is efficient by default.
  - Rate limiting is implemented (`tower_governor`).
  - CORS is configured.
- **Optimizations**:
  - Ensure `tower_http::compression::CompressionLayer` is added if bandwidth becomes a concern (not currently present in `main.rs`).

### 5. External Service Calls (S3/R2)
**Grade**: 8/10
- **Observations**:
  - S3 uploads are performed sequentially after image processing. Since it's a single file per request (avatar), parallelization isn't applicable here, but the implementation is correct.

---

## Priority Recommendations

### P0 (Critical - Immediate fix)
1. **Offload Password Hashing**:
   - **Impact**: Blocks async runtime. 1 login request can stall thousands of other requests on the same thread.
   - **Solution**: Wrap `Argon2` calls in `tokio::task::spawn_blocking`.

### P1 (High - Fix soon)
1. **Add Database Transactions**:
   - **Impact**: Potential data inconsistency (User created without tokens) and race conditions.
   - **Solution**: Use `sqlx::Transaction` for `register_user` and similar flows.

### P2 (Medium - Consider)
1. **Tune Database Pool**:
   - **Impact**: Throttling under load.
   - **Solution**: Increase `max_connections` (e.g., to 50) via env var or config.
2. **Streaming for Large Files**:
   - **Impact**: High memory usage during uploads.
   - **Solution**: For files that don't need processing, stream directly to S3. For images needing processing, consider streaming parsers if limits increase.

---

## Estimated Performance Gains
- **Offloading Hashing**: +1000% throughput during login spikes; Reduced p99 latency for *all* endpoints (not just auth) because the event loop is no longer blocked.
- **Transactions**: Improved data integrity (0% orphan records).

---

## Code Examples

### Fix: Offload Password Hashing
**File**: `src/services/auth/auth_service.rs`

**Current (Blocking):**
```rust
fn hash_password(password: &str) -> Result<String, AuthError> {
    // ... heavy computation ...
}

pub async fn register_user(...) {
    let hashed_password = hash_password(password)?; // BLOCKS!
    // ...
}
```

**Optimized (Non-blocking):**
```rust
pub async fn register_user(...) {
    let password_clone = password.to_string();
    let hashed_password = tokio::task::spawn_blocking(move || {
        hash_password(&password_clone)
    }).await
    .map_err(|_| AuthError::InternalError("Thread join error".to_string()))??;

    // ...
}
```

### Fix: Database Transactions
**File**: `src/services/auth/auth_service.rs`

**Current:**
```rust
let user = user_repository::create_user(pool, ...).await?;
token_repository::create_token(pool, ...).await?;
```

**Optimized:**
```rust
let mut tx = pool.begin().await?;
// Pass &mut tx instead of pool to repository functions (requires refactoring repos to accept Executor trait)
// Or use sqlx::query directly inside service if repos are strict
tx.commit().await?;
```
