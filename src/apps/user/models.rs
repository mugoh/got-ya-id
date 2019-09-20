//! This module holds items related to data manipulation
//! for the User Object
//!

use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use crate::apps::user::utils::validate_name;

/// User Object
/// Holds user data
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct User {
    id: u32,
    #[validate(length(min = 5), custom = "validate_name")]
    first_name: String,
    #[validate(length(min = 5), custom = "validate_name")]
    last_name: String,
    #[validate(length(min = 5), custom = "validate_name")]
    middle_name: Option<String>,
    #[validate(email(message = "%s? Email format not invented yet"))]
    email: String,
    #[validate(phone(message = "Invalid phone number"))]
    phone: Option<String>,
    is_active: bool,
    verified: bool,
}

// Default field values for User
/*
impl Default for User {
    fn default() -> User {
        User {
            is_active: true,
            verified: false,
        }
    }
}*/
