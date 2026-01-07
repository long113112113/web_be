use crate::constant::auth::MIN_PASSWORD_LENGTH;
use validator::ValidationError;

/// Custom validator for password strength requirements.
/// Returns Ok if password meets all requirements, otherwise returns ValidationError.
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(ValidationError::new("password_too_short").with_message(
            format!(
                "Password must be at least {} characters",
                MIN_PASSWORD_LENGTH
            )
            .into(),
        ));
    }

    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;

    for c in password.chars() {
        if c.is_uppercase() { has_upper = true; }
        if c.is_lowercase() { has_lower = true; }
        if c.is_numeric() { has_digit = true; }
        if !c.is_alphanumeric() { has_special = true; }

        if has_upper && has_lower && has_digit && has_special {
            return Ok(());
        }
    }

    Err(ValidationError::new("weak_password").with_message(
        "Password must contain at least one uppercase letter, one lowercase letter, one digit, and one special character".into(),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_too_short() {
        let result = validate_password_strength("Short1!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "password_too_short");
    }

    #[test]
    fn test_password_missing_uppercase() {
        let result = validate_password_strength("weakpassword1!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "weak_password");
    }

    #[test]
    fn test_password_missing_lowercase() {
        let result = validate_password_strength("WEAKPASSWORD1!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "weak_password");
    }

    #[test]
    fn test_password_missing_digit() {
        let result = validate_password_strength("WeakPassword!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "weak_password");
    }

    #[test]
    fn test_password_missing_special() {
        let result = validate_password_strength("WeakPassword1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "weak_password");
    }

    #[test]
    fn test_valid_password() {
        let result = validate_password_strength("StrongPassword1!");
        assert!(result.is_ok());
    }
}
