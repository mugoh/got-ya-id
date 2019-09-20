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
        static ref NAME_PATTERN: Regex = Regex::new("/^[A-Za-z]+$/").unwrap();
    }
    if !NAME_PATTERN.is_match(name) {
        return Err(ValidationError::new("Name should contain letters only"));
    }
    Ok(())
}
