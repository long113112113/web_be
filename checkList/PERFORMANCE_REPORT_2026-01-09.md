# Performance Analysis Report

## Executive Summary
- **Overall performance grade**: 7/10
- **Critical issues**: 1 (Sync blocking operations in Auth)
- **Optimization opportunities**: 4

The application follows a clean architecture with reasonable separation of concerns. The use of Axum, Tokio, and SQLx provides a solid high-performance foundation. However, a **critical performance bottleneck** exists in the authentication flow where CPU-intensive password hashing blocks the async runtime. Additionally, there are opportunities to optimize memory usage during file uploads and database connection management.

## Findings by Category

### 1. Async Runtime
**Grade**: 5/10
- **Issues found**:
  - **CRITICAL**: `Argon2` password hashing in `src/services/auth/auth_service.rs` is executed synchronously on the main Tokio worker thread. This stops the runtime from handling other requests during the ~50-500ms hashing process.
  - **Good**: Image metadata stripping in `src/handlers/profile.rs` is correctly offloaded using `spawn_blocking`.
- **Recommendations**:
  - Move password hashing and verification to `spawn_blocking`.

### 2. Database
**Grade**: 7/10
- **Issues**:
  - `max_connections(10)` in `src/main.rs` is likely too low for a production workload given the default Postgres settings often allow 100+.
  - `acquire_timeout(3s)` is quite aggressive and may lead to 500 errors during brief load spikes.
  - Transaction atomicity is missing in `register_user` and `refresh_access_token` (multiple steps).
  - Dynamic query building in `profile_repository` is functional but could be cleaner/safer with `QueryBuilder`.
- **Optimizations**:
  - Increase connection pool size based on available DB resources.
  - Wrap multi-step writes in transactions.

### 3. Memory Allocation & Usage
**Grade**: 6/10
- **Issues**:
  - **Multipart Uploads**: Entire files (up to 5MB) are loaded into `Vec<u8>` in memory.
  - **Image Processing**: `strip_metadata` creates a second copy of the image in memory.
  - **S3 Upload**: Uploads from the memory buffer.
  - **Impact**: 100 concurrent uploads of 5MB files would consume >1GB of RAM (input buffer + processed buffer + overhead).
- **Optimizations**:
  - While `image` crate requires buffers, limits should be strictly enforced.

### 4. HTTP Request/Response Performance
**Grade**: 8/10
- **Issues**:
  - Rate limiting burst size (5) is very low compared to the rate (60/s). This might cause false positives for legitimate concurrent client requests (e.g., page load with multiple API calls).
  - Middleware chain is lightweight (JWT only).

### 5. External Service Calls
**Grade**: 7/10
- **Issues**:
  - S3 uploads are buffered. Streaming directly from the request body to S3 would save significant memory but would make image processing (metadata stripping) harder without temporary files.

---

## Priority Recommendations

### P0 (Critical - Immediate fix)
1. **Offload Password Hashing to Blocking Pool**
   - **Impact**: Prevents one login request from stalling the entire server's ability to accept connections.
   - **Location**: `src/services/auth/auth_service.rs`

### P1 (High - Fix soon)
2. **Optimize Database Pool Configuration**
   - **Impact**: Improves throughput under load and reduces timeout errors.
   - **Location**: `src/main.rs`
3. **Wrap Auth Operations in Transactions**
   - **Impact**: Ensures data integrity (no orphan users without tokens).
   - **Location**: `src/services/auth/auth_service.rs`

### P2 (Medium - Consider)
4. **Tune Rate Limiting**
   - **Impact**: Improves UX for legitimate users making parallel requests.
   - **Location**: `src/main.rs`
5. **Optimize Dynamic Query Building**
   - **Impact**: Cleaner code and potentially better statement caching.
   - **Location**: `src/repository/profile_repository.rs`

---

## Estimated Performance Gains
- **Async Runtime Fix**: +1000% throughput during concurrent login bursts (avoids head-of-line blocking).
- **Pool Optimization**: +50-100% database throughput (depending on DB hardware).

---

## Code Examples

### 1. Fix Async Blocking (P0)

**Current Code (`src/services/auth/auth_service.rs`):**
```rust
fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        // ... blocks here
}
```

**Optimized Code:**
```rust
async fn hash_password(password: String) -> Result<String, AuthError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AuthError::HashingError(e.to_string()))
    })
    .await
    .map_err(|_| AuthError::InternalError("Task join error".to_string()))?
}
```

### 2. Database Pool Tuning (P1)

**Current Code (`src/main.rs`):**
```rust
.max_connections(10)
.acquire_timeout(Duration::from_secs(3))
```

**Optimized Code:**
```rust
.max_connections(50) // Adjust based on DB limits
.min_connections(10)
.acquire_timeout(Duration::from_secs(30)) // Give more time during spikes
```

### 3. Rate Limit Tuning (P2)

**Current Code (`src/main.rs`):**
```rust
.per_second(60)
.burst_size(5)
```

**Optimized Code:**
```rust
.per_second(60)
.burst_size(30) // Allow bursts of parallel requests from a single IP
```
