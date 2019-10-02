use lazy_static;
use regex::Regex;
use validator::ValidationError;

/// Validates name
/// - Ensures the name input is composed of alphabet characters
///  only
///
///  # Returns
///
///  ## ValidationError
/// If the validation fails
pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref NAME_PATTERN: Regex = Regex::new(r"^[a-zA-Z]+$").unwrap();
    }
    if !NAME_PATTERN.is_match(name) {
        return Err(ValidationError::new("Name should only contain letters"));
    }
    Ok(())
}

/// Validates Passwords
/// - Ensures the password inputs match a required regex pattern
///
///
///  # Returns
///
///  ## ValidationError
/// If the validation fails
pub fn validate_pass(pass: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref PASSWORD: Regex = Regex::new(r"^.{6,25}$").unwrap();
    }
    if !PASSWORD.is_match(pass) {
        return Err(ValidationError::new(
            "Password should contain:\n At least 6 characters",
        ));
    }
    Ok(())
}
