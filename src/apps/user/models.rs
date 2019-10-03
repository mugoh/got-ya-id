//! This module holds items related to data manipulation
//! for the User Object

use crate::apps::user::utils::validate_name;
use crate::diesel_cfg::{config::connect_to_db, schema::users};

use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::NaiveDateTime;
use diesel::{self, prelude::*};
use log::error;

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
    updated_at: NaiveDateTime,
    is_active: bool,
    is_verified: bool,
}

/// Temporary holds new User data
/// User Record for new User entries
#[derive(Debug, Clone, Validate, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(
        length(min = 5, message = "Make username at least 5 letters long"),
        custom = "validate_name"
    )]
    pub username: String,
    #[validate(length(min = 6, message = "Insecure password. Give it at least 6 characters"))]
    password: String,
    #[validate(email(message = "Email format not invented yet"))]
    pub email: String,
}

impl NewUser {
    /// Saves a new user record to the db
    ///
    /// # Returns
    /// User
    pub fn save(&mut self) -> Result<User, String> {
        match self.is_unique() {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "{key} Oopsy! {field} already in use",
                    key = e.0,
                    field = e.1
                ))
            }
        }
        match hash(&self.password, DEFAULT_COST) {
            Ok(h) => self.password = h,
            Err(e) => {
                error!("{}", &format!("{:?}", e));
                return Err("Failed to hash password".to_string());
            }
        };
        Ok(diesel::insert_into(users::table)
            .values(&*self) // diesel::Insertable unimplemented for &mut
            .get_result(&connect_to_db())
            .expect("Error saving user"))
    }

    /// Checks if the Email and Username given
    /// are present
    fn is_unique(&self) -> Result<(), (String, String)> {
        use crate::diesel_cfg::schema::users::dsl::*;

        let present_user = users
            .filter(email.eq(&self.email))
            .or_filter(username.eq(&self.username))
            .select((email, username))
            .get_results::<(String, String)>(&connect_to_db())
            .unwrap();

        for rec in &present_user {
            let (email_, username_) = rec;
            if email_.eq(&self.email) {
                return Err(("Email: ".to_string(), email_.to_string()));
            } else if username_.eq(&self.username) {
                return Err(("Username: ".to_string(), username_.to_string()));
            }
        }
        Ok(())
    }
}

/// Implementations for saved user records
/// These methods handle data access and manipulation
impl User {
    /// Checks the received str against the hashed
    /// user password
    ///
    /// # Returns
    ///
    /// bool: True -> Verified, False -> Fail
    fn verify_pass<'a>(&self, pass: &'a str) -> bool {
        verify(pass, &self.password).unwrap()
    }
}

/// Holds Sign-In user details
#[derive(Deserialize, Serialize, Validate)]
pub struct SignInUser {
    #[validate(email(message = "Oops! Email format not invented yet"))]
    email: Option<String>,
    username: Option<String>,
    password: String,
}
