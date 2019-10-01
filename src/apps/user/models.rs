//! This module holds items related to data manipulation
//! for the User Object

use crate::apps::user::utils::{validate_name, validate_pass};
use crate::diesel_cfg::{config::connect_to_db, schema::users};

use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use chrono::prelude::*;
use chrono::NaiveDateTime;
use diesel::{self, pg::PgConnection, prelude::*};

/// User Object
/// Holds user data
#[derive(Queryable, Debug, Clone, Validate)]
pub struct User {
    id: i32,
    pub username: String,
    pub email: String,
    password: String,
    phone: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    created_at: NaiveDateTime,
    is_active: bool,
    is_verified: bool,
}

/// Temporary holds new User data
/// User Record for new User entries
#[derive(Debug, Clone, Validate, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
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
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Option<String>,
}

impl NewUser {
    /// Saves a new user record to the db
    ///
    /// # Returns
    /// User
    pub fn save(&self) -> User {
        diesel::insert_into(users::table)
            .values(self)
            .get_result(&connect_to_db())
            .expect("Error saving user")
    }
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
