use web_be::services::auth::auth_service::validate_password;

#[test]
fn test_strong_password() {
    assert!(validate_password("Password123!").is_ok());
}

#[test]
fn test_weak_password_no_uppercase() {
    assert!(validate_password("password123!").is_err());
}

#[test]
fn test_weak_password_no_lowercase() {
    assert!(validate_password("PASSWORD123!").is_err());
}

#[test]
fn test_weak_password_no_digit() {
    assert!(validate_password("Password!").is_err());
}

#[test]
fn test_weak_password_no_special() {
    assert!(validate_password("Password123").is_err());
}

#[test]
fn test_short_password() {
    assert!(validate_password("Pass1!").is_err());
}
