# Security Audit Report

## Executive Summary
- **Date**: 2026-01-08
- **Project**: Rust Backend Web Application
- **Risk Level**: High
- **Vulnerabilities Found**: 2 High, 1 Medium, 0 Critical
- **Security Score**: 7.5/10

The application demonstrates **excellent** security practices in authentication logic, password handling, and database interactions. The use of `Argon2` for hashing, strict cookie policies, and parameterized SQL queries provides a strong foundation.

However, **High Priority** vulnerabilities were identified in the **Rate Limiting** configuration (leaving authentication endpoints exposed to brute force) and **File Upload** handling (susceptible to DoS via memory exhaustion). Addressing these will significantly elevate the security posture.

---

## Critical Vulnerabilities (CVSS 9.0+)
*None identified.*

---

## High Risk Issues (CVSS 7.0-8.9)

### 1. Lack of Rate Limiting on Public Routes
- **Severity**: HIGH
- **CVSS Score**: 7.5 (AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H)
- **Description**:
  The `tower_governor` rate limiting layer is configured in `main.rs` but only applied to the `private_routes` router (specifically `/avatar` and `/edit`). The `public_routes` router, which contains sensitive authentication endpoints (`/login`, `/register`, `/refresh-token`), **does not** have the rate limit layer applied.
- **Impact**:
  Attackers can perform:
  - Brute force attacks against user passwords.
  - Credential stuffing attacks.
  - Resource exhaustion (DoS) by flooding the login endpoint with Argon2 verification requests (which are CPU intensive).
- **Remediation**:
  Apply the rate limit layer to the public routes in `src/routes/public/auth_routes.rs` or `src/routes/public/mod.rs`.

  ```rust
  // src/routes/public/mod.rs
  pub fn public_routes(state: AppState) -> Router {
      let rate_limit_layer = tower_governor::GovernorLayer::new(
          state.rate_limit_config.clone()
      );

      Router::new()
          .nest("/auth", auth_routes::auth_routes(state))
          .layer(rate_limit_layer) // ✅ Apply protection
  }
  ```

### 2. Unbounded Memory Consumption in File Upload (DoS)
- **Severity**: HIGH
- **CVSS Score**: 7.5 (AV:N/AC:L/PR:L/UI:N/S:U/C:N/I:N/A:H)
- **Description**:
  In `src/handlers/profile.rs`, the file upload handler uses `field.bytes().await` to read the entire file content into memory **before** verifying its size.
  ```rust
  // src/handlers/profile.rs
  file_bytes = Some(
      field.bytes().await // ❌ Reads unlimited bytes into RAM
      .map_err(...)?
      .to_vec()
  );
  // Size check happens AFTER this
  if file_bytes.len() > MAX_AVATAR_SIZE { ... }
  ```
- **Impact**:
  An authenticated attacker (or unauthenticated if exposed) can send a very large file (e.g., 10GB) which the server attempts to buffer into RAM, leading to an Out-Of-Memory (OOM) crash and Denial of Service.
- **Remediation**:
  Stream the file and enforce the limit during reading, or use `axum::extract::DefaultBodyLimit` (though that applies to the whole body). Better to use a stream wrapper or check chunks.

  ```rust
  // Better approach using axum's stream constraints or manual chunking
  let mut data = Vec::new();
  while let Some(chunk) = field.chunk().await? {
      if data.len() + chunk.len() > MAX_AVATAR_SIZE {
          return Err(AppError::BadRequest("File too large"));
      }
      data.extend_from_slice(&chunk);
  }
  ```

---

## Medium Risk Issues (CVSS 4.0-6.9)

### 3. Potential CORS Misconfiguration
- **Severity**: MEDIUM
- **CVSS Score**: 4.5
- **Description**:
  The application enables `allow_credentials(true)` globally. The `CORS_ORIGINS` environment variable allows a list of origins. If a user inadvertently sets `CORS_ORIGINS=*` (or if the default behavior falls back to a wildcard in a misconfigured environment), `allow_credentials` coupled with wildcard origins is a security risk (though modern browsers block this specific combination). The risk is relying on correct `.env` configuration for security.
- **Remediation**:
  - Add validation in `Config::init` to ensure `CORS_ORIGINS` does not contain `*` if credentials are allowed.
  - Fail startup if insecure configuration is detected.

---

## Security Best Practices Compliance

### Authentication ✅
- **JWT Implementation**: ✅ **Excellent**. Uses secure validation, proper expiration, and `token_type` checks to prevent substitution attacks.
- **Password Hashing**: ✅ **Excellent**. Uses `Argon2` with `OsRng` salts.
- **Token Storage**: ✅ **Excellent**. Uses `HttpOnly`, `Secure`, `SameSite::Strict` cookies. Refresh tokens are hashed in the database (SHA-256), preventing leakage even if the DB is compromised.
- **Session Management**: ✅ **Excellent**. Implements Refresh Token Rotation.

### Input Validation ✅
- **SQL Injection**: ✅ **Safe**. Uses `sqlx::query_as!` macros for parameterized queries.
- **Data Validation**: ✅ **Good**. Uses `validator` crate for email and password complexity (uppercase, lowercase, digit, special char).
- **Image Security**: ✅ **Good**. Uses `strip_metadata` (re-encoding) to remove potential exploits in image headers/EXIF data.

### Dependencies
- **`image` Crate**: ⚠️ Caution. The `image` crate handles complex parsing and has had historical panics on malformed inputs.
  - **Mitigation**: The code correctly uses `tokio::task::spawn_blocking` to isolate the intensive work, but a panic could still abort the worker thread. Ensure the panic handler is robust.

---

## Remediation Roadmap

### Phase 1 (Immediate - 1 week)
1.  **Fix Rate Limiting**: Apply `GovernorLayer` to `public_routes` immediately to protect `/auth/login` and `/auth/register`.
2.  **Fix File Upload DoS**: Refactor `upload_avatar_handler` and `edit_profile_handler` to read the file stream in chunks and enforce `MAX_AVATAR_SIZE` incrementally.

### Phase 2 (Short-term - 1 month)
1.  **CORS Validation**: Add a check in `src/config.rs` to reject `*` in origins when credentials are supported.
2.  **Audit Logs**: Implement structured audit logging for security events (login attempts, failed uploads, profile changes). Currently, `tracing` is initialized but specific security events should be tagged.

### Phase 3 (Long-term - 3 months)
1.  **Dependency Hardening**: Consider running `cargo audit` in CI/CD pipelines.
2.  **Security Headers**: Add `tower_http::SetResponseHeaderLayer` to add `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, and `Content-Security-Policy`.

---

## Security Testing Performed
- [x] **Static Analysis**: Code review of authentication flow, database interaction, and input handling.
- [x] **Configuration Review**: Audit of `Cargo.toml`, `config.rs`, and middleware setup.
- [x] **Vulnerability Mapping**: Mapped findings to OWASP Top 10 2021.
    - A01:2021-Broken Access Control: ✅ Addressed via JWT & RBAC.
    - A03:2021-Injection: ✅ Addressed via `sqlx`.
    - A04:2021-Insecure Design: ⚠️ Rate limiting gap identified.
    - A07:2021-Identification and Authentication Failures: ⚠️ Rate limiting gap identified.
