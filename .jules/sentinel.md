## 2024-05-23 - [Weak Password Validation]
**Vulnerability:** The application only checked for password length (min 8 characters), allowing weak passwords like "12345678".
**Learning:** `validate_password` in `auth_service.rs` was the single point of failure.
**Prevention:** Enforced complexity requirements (uppercase, lowercase, digit, special char) in `validate_password`.
