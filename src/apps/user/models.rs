//! This module holds items related to data manipulation
//! for the User Object

use crate::apps::user::utils::validate_name;
use crate::config::config;
use crate::diesel_cfg::{config::connect_to_db, schema::users};

use std::error;

use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::NaiveDateTime;
use diesel::{self, prelude::*};
use log::error;

use jsonwebtoken as jwt;
use jwt::{encode, Header};

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

impl SignInUser {
    /// Signs in User
    ///
    /// - Checks if user is registered
    pub fn sign_in(&self) -> Result<String, diesel::result::Error> {
        use crate::diesel_cfg::schema::users::dsl::*;

        let (key, identity) = if self.email.is_some() {
            ("email", &self.email)
        } else {
            ("username", &self.username)
        };

        let query = match &key {
            &"email" => users
                .filter(email.eq(self.email.clone().unwrap()))
                .select(email)
                .first::<String>(&connect_to_db()),
            _ => users
                .filter(username.eq(self.username.clone().unwrap()))
                .select(username)
                .first::<String>(&connect_to_db()),
        };
        query
    }

    /// Verifies the given Sign In detail contains
    /// either a Username or an Email
    ///
    /// # Returns
    /// bool
    ///
    /// - True: For at least 1 is_some() true evaluation
    /// - False: is_none() for both email and username
    pub fn has_credentials(&self) -> bool {
        vec![&self.username, &self.email]
            .iter()
            .all(|&x| x.is_none())
    }

    /// Creates an authorization token encoded with the
    /// given user detail
    pub fn create_token(&self, user_cred: &String) -> Result<String, Box<dyn error::Error>> {
        let payload = Claims {
            company: user_cred.to_owned(),
            exp: 10000000000,
        };

        // ENV Configuration
        let conf = config::get_env_config().unwrap_or_else(|err| {
            eprintln!("Error: Missing required ENV Variable\n{:#?}", err);
            std::process::exit(78);
        });
        let key = &conf.secret_key;

        let header = Header::default();

        match encode(&header, &payload, key.as_ref()) {
            Ok(t) => Ok(t),
            Err(e) => Result::Err(Box::new(e)),
        }
    }
}

/// JWT Authorization
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub company: String,
    pub exp: usize,
}
