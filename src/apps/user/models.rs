//! This module holds items related to data manipulation
//! for the User Object

use super::utils::{naive_date_format, validate_email, validate_name};

use std::borrow::Cow;

use crate::apps::auth::validate;
use crate::apps::profiles::models::{NewProfile, Profile};
use crate::config::config;
use crate::diesel_cfg::{config::connect_to_db, schema::users};

use std::error;

use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{prelude::*, Duration, NaiveDateTime};
use diesel::{self, prelude::*};
use log::{debug, error};

use jsonwebtoken as jwt;
use jwt::{encode, Header};

/// User Object
/// Holds user data
#[derive(Queryable, Serialize, AsChangeset, Deserialize, Identifiable, Debug, Clone, Validate)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_deserializing)]
    password: String,
    #[serde(with = "naive_date_format")]
    created_at: NaiveDateTime,
    #[serde(with = "naive_date_format")]
    updated_at: NaiveDateTime,
    pub is_active: bool,
    pub is_verified: bool,
}

/// Temporary holds new User data
/// User Record for new User entries
#[derive(Debug, Clone, Validate, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
#[serde(deny_unknown_fields)]
pub struct NewUser<'b> {
    #[validate(
        length(min = 5, message = "Make username at least 5 letters long"),
        custom = "validate_name"
    )]
    pub username: Cow<'b, str>,
    #[validate(length(min = 6, message = "Insecure password. Give it at least 6 characters"))]
    password: Cow<'b, str>,
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Cow<'b, str>,
}

/// Holds data passed on Password-reset request
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct PassResetData<'a> {
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Cow<'a, str>,
}

/// Holds Account Password reset data
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPassData {
    #[validate(length(min = 5, message = "Give your password at least 5 characters"))]
    pub password: String,
    #[validate(must_match = "password")] // Can't give error message given on failed match
    password_conf: String,
}

impl<'a> NewUser<'a> {
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
            Ok(h) => self.password = Cow::Owned(h),
            Err(e) => {
                error!("{}", &format!("{:?}", e));
                return Err("Failed to hash password".to_string());
            }
        };
        let usr = diesel::insert_into(users::table)
            .values(&*self) // diesel::Insertable unimplemented for &mut
            .get_result::<User>(&connect_to_db())
            .expect("Error saving user");
        NewProfile::new(usr.id, None)?;
        Ok(usr)
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
    pub fn verify_pass<'a>(&self, pass: &'a str) -> Result<bool, ()> {
        verify(pass, &self.password).map_err(|e| debug!("{:?}", e))
    }

    /// Creates an authorization token encoded with the
    /// given user detail
    pub fn create_token(&self, user_cred: &String) -> Result<String, Box<dyn error::Error>> {
        let payload = Claims {
            company: user_cred.to_owned(),
            exp: (Utc::now() + Duration::seconds(75)).timestamp() as usize,
            sub: "login".to_string(),
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

    /// Decodes the auth token representing a user
    /// to return an user object with a verified account
    pub fn verify_user(user_key: &String) -> Result<User, Box<dyn error::Error>> {
        use crate::diesel_cfg::schema::users::dsl::*;
        let user = match validate::decode_auth_token(user_key) {
            Ok(user_detail) => user_detail.company,
            Err(e) => {
                // return (status code, e)
                return Err(e.into());
            }
        };
        let user = diesel::update(users.filter(email.eq(&user)))
            .set(is_verified.eq(true))
            .get_result::<User>(&connect_to_db())
            .unwrap();

        Ok(user)
    }

    /// Alters the existing account password to match
    /// the string passed as a new password.
    pub fn reset_pass(token: &String, new_password: &String) -> Result<(), Box<dyn error::Error>> {
        use crate::diesel_cfg::schema::users::dsl::*;

        let user = match validate::decode_auth_token(token) {
            Ok(usr) => usr.company,
            Err(e) => return Err(e.into()),
        };
        let pass_hash = match hash(new_password, DEFAULT_COST) {
            Ok(h) => h,
            Err(e) => {
                error!("{}", &format!("{:?}", e));
                return Err(e.into());
            }
        };
        diesel::update(users.filter(email.eq(&user)))
            .set(password.eq(pass_hash))
            .get_result::<User>(&connect_to_db())
            .unwrap();
        Ok(())
    }

    /// Finds a user by email
    ///
    /// # Returns
    ///
    /// ## Result
    /// OK -> User object that matches the given email
    /// ERR -> String
    pub fn find_by_email<'a>(given_email: &'a str) -> Result<Vec<User>, String> {
        use crate::diesel_cfg::schema::users::dsl::{email, users};

        let user = users
            .filter(email.eq(given_email))
            .load::<User>(&connect_to_db())
            .unwrap();
        match user.is_empty() {
            false => Ok(user),
            _ => Err(format!("User of email {} non-existent", given_email).into()),
        }
    }

    /// Finds a User by Primary key
    ///
    /// # Returns
    /// The user object and corresponding Profile
    ///
    pub fn find_by_pk<'a>(pk: i32) -> Result<(User, Option<Profile<'a>>), Box<dyn error::Error>> {
        //
        use crate::diesel_cfg::schema::users::dsl::*;
        let user = users.find(pk).get_result::<User>(&connect_to_db())?;
        let mut usr_profile = Profile::belonging_to(&user).load::<Profile>(&connect_to_db())?;
        if usr_profile.is_empty() {
            Err(format!("User of ID {id} non existent", id = pk))?
        }
        Ok((user, usr_profile.pop()))
    }

    /// Retrieves all existing User profiles
    ///
    /// ``Moved to profiles::retrieve_all (separate retrieval of profiles)``
    ///
    /// # Arguments
    ///
    /// ## with_profile: Option<u8>
    ///  Return each User Object with its corresponding Profile
    ///     WARNING ->
    /// If Some(u8), a second query will be done for ALL user profiles
    pub fn retrieve_all<'a>(with_profile: Option<u8>) -> Result<Vec<User>, Box<dyn error::Error>> {
        use crate::diesel_cfg::schema::users::dsl::*;
        let user_vec = users.load::<User>(&connect_to_db()).unwrap();

        match with_profile {
            Some(_) => {
                Err("Unimplemented")?;
                let mut res: std::collections::HashMap<usize, (&User, Profile)> =
                    std::collections::HashMap::new();
                for (i, usr) in user_vec.iter().enumerate() {
                    let profile = Profile::belonging_to(usr)
                        .first::<Profile<'a>>(&connect_to_db())
                        .unwrap();
                    res.insert(i, (usr, profile));
                }
            }
            _ => (),
        }
        Ok(user_vec)
    }

    /// Alters an account activation status
    /// Activates or Deactivates a User account
    pub fn alter_activation_status(&self) -> Result<User, Box<dyn error::Error>> {
        //
        use crate::diesel_cfg::schema::users::dsl::*;
        Ok(diesel::update(&*self)
            .set(is_active.eq(!self.is_active))
            .get_result::<User>(&connect_to_db())?)
    }
}

/// Holds Sign-In user details
#[derive(Deserialize, Serialize, Validate)]
pub struct SignInUser<'a> {
    // #[validate(email(message = "Oops! Email format not invented yet"))]

    // Email validation Panicks with :: ->
    /* the trait bound `std::borrow::Cow<'_, str>: std::convert::From<&std::borrow::Cow<'_, str>>` is not satisfied */
    #[validate(custom = "validate_email")]
    email: Option<Cow<'a, str>>,
    username: Option<Cow<'a, str>>,
    password: Cow<'a, str>,
}

impl<'a> SignInUser<'a> {
    /// Signs in User
    ///
    /// - Checks if user is registered
    pub fn sign_in(&self) -> Result<Vec<User>, diesel::result::Error> {
        use crate::diesel_cfg::schema::users::dsl::*;

        let (key, identity) = if self.email.is_some() {
            ("email", &self.email)
        } else {
            ("username", &self.username)
        };

        let query = match &key {
            &"email" => users
                .filter(email.eq(identity.clone().unwrap()))
                // .select(email)
                .load::<User>(&connect_to_db()),
            _ => users
                .filter(username.eq(identity.clone().unwrap()))
                // .select(username)
                .load::<User>(&connect_to_db()),
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

    /// Retreives the password field given on sign in
    pub fn get_password(&self) -> &str {
        self.password.as_ref()
    }
}

/// Holds JWT Authorization Claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub company: String,
    pub exp: usize,
    sub: String,
}

/// Json Request data with Email field only
#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserEmail<'a> {
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Cow<'a, str>,
}
