//! Custom input validators
use regex::Regex;
use validator::ValidationError;

/// Validates location names
/// - Ensures the name input is composed of only alphaneumeric characters
pub fn validate_location_name(name: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref NAME_PATTERN: Regex = Regex::new(r"^[a-zA-Z0-9 -`_]+$").unwrap();
    }
    if !NAME_PATTERN.is_match(name) {
        return Err(ValidationError::new(
            "should just have letters, digits or -_",
        ));
    }
    Ok(())
}

/// Static validation regex instances
pub mod regexes {
    use super::Regex;
    lazy_static! {
        pub static ref ALPHA_REGEX: Regex = Regex::new(r"^[a-zA-Z ]+$").unwrap();
    }
    lazy_static! {
        pub static ref LOCATION_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9 -`_]+$").unwrap();
    }
}

/// Validates alphabetic regex
/// - Ensures the name input is composed of only alphaneumeric characters
pub fn validate_alpha_regex(name: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref PATTERN: Regex = Regex::new(r"^[a-zA-Z]+$").unwrap();
    }
    if !PATTERN.is_match(name) {
        return Err(ValidationError::new("should just have letters"));
    }
    Ok(())
}

pub fn validate_str_len(name: &str) -> Result<(), ValidationError> {
    let min: usize = 3;
    let max: usize = 255;
    if name.len() < min || name.len() > max {
        return Err(ValidationError::new(
            "should be between 3 and 255 characters",
        ));
    }
    Ok(())
}
