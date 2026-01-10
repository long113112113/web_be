# Security Audit Report

**Date:** 2026-01-10
**Target:** Web Backend (Rust/Axum)
**Auditor:** Jules (Senior Security Engineer)

## Executive Summary
- **Risk Level:** High
- **Vulnerabilities Found:** 6 (1 Critical, 1 High, 3 Medium, 1 Low)
- **Security Score:** 7/10

The application demonstrates a solid foundation in security, particularly in modern authentication practices (Argon2, HttpOnly cookies) and memory safety (Rust). However, significant risks exist in **Rate Limiting configuration** and **Transactional Integrity** during registration.

## Critical Vulnerabilities (CVSS 9.0+)

*None identified.*

## High Risk Issues (CVSS 7.0-8.9)

### 1. Ineffective Rate Limiting for Sensitive Endpoints
- **Severity:** HIGH
- **CVSS:** 7.5 (AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H)
- **Location:** `src/main.rs`, `src/routes/public/auth_routes.rs`
- **Description:** The rate limiting configuration allows **60 requests per second** (burst 5) per IP. This configuration is shared across all protected routes, including `/login` and `/register`.
- **Impact:** An attacker can perform brute-force attacks against user accounts at a rate of 3,600 attempts per minute per IP, rendering the protection ineffective against online password guessing.
- **Remediation:**
  - Create a separate, stricter rate limit configuration for authentication endpoints.
  - Recommended limit: 5 requests per minute for login/registration.

  ```rust
  // Suggested Fix in main.rs
  let auth_rate_limit = GovernorConfigBuilder::default()
      .per_second(1) // or per_minute(5)
      .burst_size(2)
      .finish()
      .unwrap();
  ```

## Medium Risk Issues (CVSS 4.0-6.9)

### 2. Lack of Transactional Integrity in Registration
- **Severity:** MEDIUM
- **CVSS:** 5.0 (AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:P/A:N)
- **Location:** `src/services/auth/auth_service.rs`
- **Description:** The `register_user` function performs two distinct database operations: `create_user` and `create_token`. If `create_token` fails (e.g., DB connection blip), the user is created but cannot log in (no refresh token), and the client receives an error.
- **Impact:** Inconsistent database state (orphaned user records).
- **Remediation:** Wrap the operations in a database transaction.
  ```rust
  let mut tx = pool.begin().await?;
  // perform operations using &mut tx
  tx.commit().await?;
  ```

### 3. User Enumeration via Error Messages
- **Severity:** MEDIUM
- **CVSS:** 4.3 (AV:N/AC:L/PR:N/UI:N/S:U/C:L/I:N/A:N)
- **Location:** `src/services/auth/auth_service.rs`
- **Description:** The registration endpoint returns `AuthError::EmailAlreadyExists` if the email is taken.
- **Impact:** Allows attackers to harvest valid email addresses from the system.
- **Remediation:** Return a generic message like "If the email is valid, a registration link has been sent" or ensure the error time and response are identical to a successful registration (difficult). Alternatively, accept this risk as a business trade-off for better UX.

### 4. Lack of Automated Security Auditing in CI/CD
- **Severity:** MEDIUM
- **CVSS:** 5.0
- **Location:** Project Root
- **Description:** `cargo audit` is not integrated into the build process/CI.
- **Impact:** Vulnerable dependencies may be introduced and go unnoticed.
- **Remediation:** Add `cargo-audit` to the CI pipeline.

## Low Risk Issues (CVSS 0.1-3.9)

### 5. Dependency on Image Format Guessing
- **Severity:** LOW
- **Location:** `src/utils/image.rs`
- **Description:** The `strip_metadata` function relies on `image` crate's format guessing. While generally robust, complex parsing logic is a common attack vector.
- **Remediation:** Ensure the `image` crate is kept up-to-date. Consider limiting the input stream size explicitly before parsing (already mitigated by body limit).

---

## Security Best Practices Compliance

### Authentication ✅
- **JWT Implementation:** ✅ Excellent. `token_type` checks prevent substitution.
- **Password Hashing:** ✅ Argon2 with `OsRng` salt is state-of-the-art.
- **Token Storage:** ✅ HttpOnly, Secure, SameSite=Strict cookies.
- **Refresh Token Rotation:** ✅ Implemented properly with hash verification.

### Authorization ✅
- **Access Control:** ✅ Endpoints protected via `Extension(Claims)` extraction.
- **CORS:** ✅ Explicit origins, wildcard rejected with credentials.

### Input Validation ✅
- **SQL Injection:** ✅ Prevented via `sqlx::query_as!` macros.
- **Data Validation:** ✅ `validator` crate used for email/passwords.

### File Upload Security ✅
- **Type Validation:** ✅ Content-Type checked and image re-encoded.
- **Filename:** ✅ Random UUIDs used.
- **Storage:** ✅ Offloaded to R2 (Cloudflare).

---

## Remediation Roadmap

### Phase 1 (Immediate - 1 week)
1. **Fix Rate Limiting:** Implement strict rate limits for `/login` and `/register`.
2. **Add Transactions:** Refactor `register_user` to use `sqlx::Transaction`.

### Phase 2 (Short-term - 1 month)
1. **CI/CD Security:** Configure GitHub Actions (or equivalent) to run `cargo audit` and `cargo clippy`.
2. **Review Logs:** Ensure `tracing` configuration does not log sensitive data in production.

## Security Testing Performed
- [x] **Static Analysis:** Code review of auth patterns, SQL queries, and configuration.
- [x] **Configuration Review:** CORS, Cookies, Environment variables.
- [x] **Logic Review:** JWT handling, file uploads, state transitions.
- [ ] **Dynamic Analysis:** (Not performed in this static audit context).
