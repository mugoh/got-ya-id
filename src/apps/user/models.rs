//! This module holds items related to data manipulation
//! for the User Object
//!

use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use crate::apps::user::utils::{validate_name, validate_pass};

/// User Object
/// Holds user data
#[derive(Debug, Clone, Queryable, Validate, Serialize, Deserialize)]
pub struct User {
    #[validate(length(min = 5), custom = "validate_name")]
    pub username: Option<String>,
    #[validate(
        length(min = 8, message = "Password should be at least 8 characters long"),
        custom = "validate_pass"
    )]
    password: Option<String>,
    #[validate(
        length(
            min = 5,
            message = "Name should be at least 5 characters and contain letters only"
        ),
        custom = "validate_name"
    )]
    first_name: Option<String>,
    #[validate(
        length(
            min = 5,
            message = "Name should be at least 5 characters and contain letters only"
        ),
        custom = "validate_name"
    )]
    last_name: Option<String>,
    #[validate(
        length(
            min = 5,
            message = "Name should be at least 5 characters and contain letters only"
        ),
        custom = "validate_name"
    )]
    middle_name: Option<String>,
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Option<String>,
    #[validate(length(min = 5, code = "phone", message = "Invalid phone number"))]
    phone: Option<String>,
}
/*
/// Default field values for User
impl Default for User {
    fn default() -> User {
        User {
            username: Some("".to_string()),
            first_name: Some("".to_string()),
            last_name: Some("".to_string()),
            middle_name: Some("".to_string()),
            email: Some("".to_string()),
            phone: Some("".to_string()),
            password: Some("".to_string()),
            is_active: Some(true),
            is_verified: Some(false),
        }
    }
}
*/
