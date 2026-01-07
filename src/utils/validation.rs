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

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_digit || !has_special {
        return Err(ValidationError::new("weak_password").with_message(
            "Password must contain at least one uppercase letter, one lowercase letter, one digit, and one special character".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_strength_success() {
        assert!(validate_password_strength("StrongP@ss1").is_ok());
        assert!(validate_password_strength("AnotherStro0ng!").is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let result = validate_password_strength("Short1!");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "password_too_short");
    }

    #[test]
    fn test_validate_password_strength_complexity() {
        // Missing uppercase
        let res = validate_password_strength("weakp@ss1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "weak_password");

        // Missing lowercase
        let res = validate_password_strength("WEAKP@SS1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "weak_password");

        // Missing digit
        let res = validate_password_strength("WeakP@ssword");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "weak_password");

        // Missing special char
        let res = validate_password_strength("WeakPass123");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code, "weak_password");
    }
}
