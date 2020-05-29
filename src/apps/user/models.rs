//! This module holds items related to data manipulation
//! for the User Object

use super::utils::{from_timestamp, validate_email, validate_name};

use std::borrow::Cow;

use crate::{
    apps::{
        auth::validate::{self, Claims},
        email::models::Email,
        profiles::models::{Avatar, NewProfile, Profile},
    },
    config::configs as config,
    core::py_interface::remove_py_mod,
    diesel_cfg::{
        config::connect_to_db,
        schema::{emails as emails_table, oath_users, refresh_tokens, users},
    },
    errors::error::ResError,
};

use std::{env, error::Error as stdError};

use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

use actix_web::error::{Error, ErrorForbidden, ErrorInternalServerError};
use actix_web::HttpRequest;

use actix_web::http::header::Header as acHeader;
use actix_web_httpauth::headers::authorization::Authorization;
use actix_web_httpauth::headers::authorization::Bearer;

use bcrypt::{hash, verify, DEFAULT_COST};

use chrono::{prelude::*, Duration, NaiveDateTime};
use diesel::{self, prelude::*};

use jsonwebtoken as jwt;
use jwt::{encode, Header};

use url::Url;

/// User Object
/// Holds user data
#[derive(Queryable, Serialize, AsChangeset, Deserialize, Identifiable, Validate)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_deserializing)]
    password: Option<String>,
    #[serde(deserialize_with = "from_timestamp")]
    pub created_at: NaiveDateTime,
    #[serde(deserialize_with = "from_timestamp")]
    updated_at: NaiveDateTime,
    pub is_active: bool,
    pub is_verified: bool,
    pub social_id: Option<String>,
    pub social_account_verified: bool,
    pub access_level: i32,
}

/// Temporary holds new User data
/// User Record for new User entries
#[derive(Clone, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser<'b> {
    pub username: Cow<'b, str>,
    pub password: Cow<'b, str>,
    pub access_level: Option<i32>,
}

/// The model representing the user
/// data used in registering new users.
///
/// This structure only adds `email` onto
/// the `NewUser` struct.
#[derive(Clone, Validate, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewJsonUser<'b> {
    #[validate(
        length(min = 5, message = "Make username at least 5 letters long"),
        custom = "validate_name"
    )]
    pub username: Cow<'b, str>,
    #[validate(length(min = 6, message = "Insecure password. Give it at least 6 characters"))]
    pub password: Cow<'b, str>,
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Cow<'b, str>,
    pub access_level: Option<i32>,
}

/// Holds Sign-In user details
#[derive(Deserialize, Serialize, Validate)]
pub struct SignInUser<'a> {
    // #[validate(email(message = "Oops! Email format not invented yet"))]

    // Email validation Panicks with :: ->
    /* the trait bound `std::borrow::Cow<'_, str>: std::convert::From<&std::borrow::Cow<'_, str>>` is not satisfied */
    #[validate(custom = "validate_email")]
    pub email: Option<Cow<'a, str>>,
    username: Option<Cow<'a, str>>,
    password: Cow<'a, str>,
}

/// Holds user email passed in email-only JSON requests
#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserEmail<'a> {
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Cow<'a, str>,
}

/// Holds Account Password reset data
#[derive(Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPassData {
    #[validate(length(min = 5, message = "Give your password at least 5 characters"))]
    pub password: String,
    #[validate(must_match = "password")] // Can't give error message given on failed match
    pub password_conf: String,
}

/// Oauth Query Params Struct extractor
#[derive(Deserialize)]
pub struct OauthInfo {
    pub code: String,
    pub state: String,
}

/// App Data extractor
///
/// Holds the Oauth Client
pub struct OClient {
    pub client: oauth2::basic::BasicClient,
}

/// The Refresh tokens Queryable model
#[derive(Queryable, Serialize, Deserialize, Identifiable)]
#[table_name = "refresh_tokens"]
pub struct Reftoken {
    id: i64,
    body: String,
    valid: bool,
}

/// The Refresh Tokens Insertable model
#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "refresh_tokens"]
pub struct NewRfToken<'a> {
    pub body: Cow<'a, str>,
}

type Tokens = (String, String);

pub enum AccessLevel {
    Admin,
    Moderator,
    Usualuser,
}

#[derive(Validate, Deserialize, Serialize)]
pub struct NewUserLevel<'a> {
    /// Email of the account whose level is to be altered
    #[validate(email(message = "Email format not invented yet"))]
    pub email: Cow<'a, str>,
    /// User access level to change to
    #[validate(range(min = 0, max = 2))]
    new_level: i32,
}

impl<'a> NewUser<'a> {
    /// Saves a new user record to the db
    ///
    /// # Returns
    /// User
    pub fn save(&mut self, new_email: &str) -> Result<User, Box<dyn stdError>> {
        use crate::diesel_cfg::schema::emails::*;

        match self.is_unique(new_email) {
            Ok(_) => (),
            Err(e) => {
                return Err(format!(
                    "{key} Oopsy! {field} already in use",
                    key = e.0,
                    field = e.1
                )
                .into())
            }
        }
        match hash(&self.password, DEFAULT_COST) {
            Ok(h) => self.password = Cow::Owned(h),
            Err(e) => {
                debug!("{}", &format!("{:?}", e));
                return Err(format!("Failed to hash password: {}", e).into());
            }
        };
        let usr = diesel::insert_into(users::table)
            .values(&*self) // diesel::Insertable unimplemented for &mut
            .get_result::<User>(&connect_to_db())?;

        let email_data = (email.eq(new_email), user_id.eq(usr.id), active.eq(true));

        diesel::insert_into(emails_table::table)
            .values(&email_data)
            .load::<Email>(&connect_to_db())?;
        NewProfile::new(usr.id, None)?;
        Ok(usr)
    }

    /// Checks if the Email and Username given
    /// are present
    fn is_unique(&self, new_email: &str) -> Result<(), (String, String)> {
        use crate::diesel_cfg::schema::emails::dsl::{email, emails};
        use crate::diesel_cfg::schema::users::dsl::*;

        let present_usernames = users
            .filter(username.eq(&self.username))
            .load::<User>(&connect_to_db())
            .unwrap();

        for user_ in present_usernames {
            if user_.username.eq(&self.username) {
                return Err(("Username: ".to_string(), user_.username));
            }
        }

        let mut email_ = emails
            .filter(email.eq(new_email))
            .load::<Email>(&connect_to_db())
            .unwrap();

        if !email_.is_empty() {
            return Err(("Email: ".to_string(), email_.pop().unwrap().email));
        }
        Ok(())
    }
}

impl<'b> NewJsonUser<'b> {
    ///   Created a NewUser, which is insertable, from JsonUser.
    pub fn into_savable(&self) -> NewUser {
        NewUser {
            username: Cow::Borrowed(&self.username),
            password: Cow::Borrowed(&self.password),
            access_level: self.access_level,
        }
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
    pub async fn verify_pass(&self, pass: &str) -> Result<bool, String> {
        verify(pass, &self.password.as_ref().unwrap()).map_err(|e| {
            debug!("{:?}", e);
            e.to_string()
        })
    }

    /// Gives the Active email of a User
    pub fn email(&self) -> String {
        use crate::diesel_cfg::schema::emails::{active, email};

        Email::belonging_to(self)
            .filter(active.eq(true))
            .select(email)
            .get_result::<String>(&connect_to_db())
            .unwrap()
    }

    /// Creates an authorization token encoded with the
    /// given user detail
    ///
    /// The cred used is the user email
    pub fn create_token(
        user_cred: &str,
        duration_min: Option<i64>,
        issuer: String,
    ) -> Result<String, ResError> {
        let dur = if let Some(time) = duration_min {
            time
        } else {
            120 // Use env variable
        };
        let payload = Claims {
            sub: user_cred.to_owned(),
            iat: (Utc::now()).timestamp() as usize,
            exp: (Utc::now() + Duration::minutes(dur)).timestamp() as usize,
            iss: issuer,
        };

        // ENV Configuration
        let conf = config::get_env_config().unwrap_or_else(|err| {
            eprintln!("Error: Missing required ENV Variable\n{:#?}", err);
            std::process::exit(78);
        });
        let key = &conf.secret_key;

        let header = Header::default();

        Ok(encode(&header, &payload, key.as_ref())?)
    }

    /// Decodes the auth token representing a user
    /// to return an user object with a verified account
    pub fn verify_user(user_key: &str) -> Result<User, Box<dyn stdError>> {
        use crate::diesel_cfg::schema::emails::dsl::{email, emails, verified};
        use crate::diesel_cfg::schema::users::dsl::*;

        let user = match validate::decode_auth_token(user_key, Some("verification".to_owned())) {
            Ok(user_detail) => user_detail.sub,
            Err(e) => {
                return Err(e.into());
            }
        };

        // While at it, set email as verified too.
        let em = diesel::update(emails.filter(email.eq(user)))
            .set(verified.eq(true))
            .get_result::<Email>(&connect_to_db())?;

        let user = diesel::update(users.find(em.user_id))
            .set(is_verified.eq(true))
            .get_result::<User>(&connect_to_db())?;

        Ok(user)
    }

    /// Alters the existing account password to match
    /// the string passed as a new password.
    pub fn reset_pass(token: &str, new_password: &str) -> Result<(), Box<dyn stdError>> {
        use crate::diesel_cfg::schema::emails::dsl::{email, emails, user_id};
        use crate::diesel_cfg::schema::users::dsl::*;

        let user = match validate::decode_auth_token(token, Some("password_reset".to_string())) {
            Ok(usr) => usr.sub,
            Err(e) => return Err(e.into()),
        };
        let pass_hash = match hash(new_password, DEFAULT_COST) {
            Ok(h) => h,
            Err(e) => return Err(e.into()),
        };

        let uid = emails
            .filter(email.eq(&user))
            .select(user_id)
            .get_result::<i32>(&connect_to_db())?;

        diesel::update(users.find(uid))
            .set(password.eq(pass_hash))
            .get_result::<User>(&connect_to_db())?;
        Ok(())
    }

    /// Finds a user by email
    ///
    /// # Returns
    ///
    /// ## Result
    /// OK -> User object that matches the given email
    /// ERR -> String
    pub fn find_by_email(given_email: &str) -> Result<Vec<User>, String> {
        match Email::load_user(given_email) {
            Err(_) => Err(format!("User of email {} non-existent", given_email)),
            Ok(user) => Ok(user),
        }
    }
    /// Finds a User by Primary key
    ///
    /// # Returns
    /// The user object and corresponding Profile
    ///
    /// # Arguments
    ///
    /// * `pk`  - User primary key
    /// * `include_profile` - If supplied, return a tuple of the user and the user profile. Returns the user instance only if None
    ///
    pub fn find_by_pk<'a>(
        pk: i32,
        include_profile: Option<i32>,
    ) -> Result<(User, Option<Profile<'a>>), Box<dyn stdError>> {
        use crate::diesel_cfg::schema::users::dsl::*;
        let user = users.find(pk).get_result::<User>(&connect_to_db())?;

        if include_profile.is_none() {
            return Ok((user, None));
        }

        let mut usr_profile = Profile::belonging_to(&user).load::<Profile>(&connect_to_db())?;
        if usr_profile.is_empty() {
            return Err(format!("User of ID {id} non existent", id = pk).into());
        }
        Ok((user, usr_profile.pop()))
    }

    /// Searches for the User and (optionally) their Profile
    /// using the PK.
    ///
    /// Same as User::find_by_pk but required authentication.
    /// This should be in response to a http request
    pub fn find_by_pk_authenticated(
        pk: i32,
        include_profile: Option<i32>,
        auth_header: &HttpRequest,
    ) -> Result<(User, Option<Profile>), ResError> {
        let user = Self::from_token(auth_header)?;
        if user.id != pk {
            return Err(ResError::unauthorized());
        }

        if include_profile.is_none() {
            return Ok((user, None));
        }

        let usr_profile = Profile::belonging_to(&user).first::<Profile>(&connect_to_db())?;
        Ok((user, Some(usr_profile)))
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
    pub fn retrieve_all(with_profile: Option<u8>) -> Result<Vec<User>, Box<dyn stdError>> {
        use crate::diesel_cfg::schema::users::dsl::*;
        let user_vec = users.load::<User>(&connect_to_db()).unwrap();

        if with_profile.is_some() {
            return Err("Unimplemented".into());
            /*
            let mut res: std::collections::HashMap<usize, (&User, Profile)> =
                std::collections::HashMap::new();
            for (i, usr) in user_vec.iter().enumerate() {
                let profile = Profile::belonging_to(usr)
                    .first::<Profile>(&connect_to_db())
                    .unwrap();
                res.insert(i, (usr, profile));
            }
            */
        }
        Ok(user_vec)
    }

    /// Alters an account activation status
    /// Activates or Deactivates a User account
    pub fn alter_activation_status(&self) -> Result<User, ResError> {
        use crate::diesel_cfg::schema::users::dsl::is_active;
        Ok(diesel::update(&*self)
            .set(is_active.eq(!self.is_active))
            .get_result::<User>(&connect_to_db())?)
    }

    /// Alters the avatar table associated with the user profile
    /// to match the given url field
    pub fn save_avatar<'b>(&self, avatar_url: &'b str) -> Result<Avatar, Box<dyn stdError>> {
        use crate::diesel_cfg::schema::avatars::dsl::*;

        let avatar = Avatar::belonging_to(self).load::<Avatar>(&connect_to_db())?;

        // Delete old avatar.
        // Avatars in the got_ya_id storage have a got_ya_id substring.
        // This should differentiate them from oauth2 profile avatars
        let old_url = &avatar[0].url;

        if old_url.is_some() && old_url.clone().unwrap().contains("got_ya_id") {
            let compl = Url::parse(&old_url.clone().unwrap())?;
            let s: Vec<&str> = compl
                .path_segments()
                .unwrap()
                .filter(|a| a.contains('.'))
                .collect();
            let ss = s[0].split('.').collect::<Vec<&str>>()[0];

            let delete_res = remove_old_url(ss).expect("Delete failed");
            info!("File deleted : {}", delete_res);
        }

        Ok(diesel::update(&avatar[0])
            .set(url.eq(avatar_url))
            .get_result::<Avatar>(&connect_to_db())?)
    }

    /// Retrieves the Avatar belonging to the user instance
    pub fn get_avatar(&self) -> Result<Option<Avatar>, diesel::result::Error> {
        Ok(Avatar::belonging_to(self)
            .load::<Avatar>(&connect_to_db())?
            .pop())
    }

    /// Change the access level of a given user
    /// The account being used to change the user level must be of
    /// a higher level or equal to the one requested
    pub fn alter_access_level(level: &NewUserLevel, auth_tk: &str) -> Result<User, String> {
        use crate::diesel_cfg::schema::emails::dsl::{email, emails, user_id};
        use crate::diesel_cfg::schema::users::dsl::*;

        let grant_email = match validate::decode_auth_token(auth_tk, Some("auth".into())) {
            Ok(claims) => claims.sub,
            Err(e) => return Err(e.to_string()),
        };
        let granter_result = Email::load_user(&grant_email);
        let granter = if let Err(e) = granter_result {
            return Err(e.to_string());
        } else {
            granter_result.unwrap()
        };

        let granter = if !granter.is_empty() {
            &granter[0]
        } else {
            return Err("Invalid token. Problem finding user".into());
        };

        if level.new_level < granter.access_level
            || granter.access_level == AccessLevel::Usualuser as i32
        {
            return Err("Oopsy! You are not allowed to do that".into());
        }

        let uid = match emails
            .filter(email.eq(&level.email))
            .select(user_id)
            .get_result::<i32>(&connect_to_db())
        {
            Ok(uid) => uid,
            Err(e) => return Err(e.to_string()),
        };
        let user_result = diesel::update(users.find(uid))
            .set(access_level.eq(level.new_level))
            .get_result::<User>(&connect_to_db());
        match user_result {
            Ok(user) => Ok(user),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Attempts to extract and decode and Authentication Bearer Header from the request.
    ///
    /// The issuer used in decoding is **"auth"**, for authentication access
    /// purposed only
    ///
    /// Returns an Error if the Header is missing or decode fails
    pub fn decode_auth_header(auth_header: &HttpRequest) -> Result<(), ResError> {
        let auth = User::extract_auth_header(auth_header)?;
        let auth_tk = &auth.split(' ').collect::<Vec<&str>>()[1];

        validate::decode_auth_token(auth_tk, Some("auth".into()))?;

        Ok(())
    }

    /// Gives the User whose email matches the subject of the decoded
    /// authorization token
    pub fn from_token(auth_header: &HttpRequest) -> Result<Self, ResError> {
        let auth = User::extract_auth_header(auth_header)?;
        let auth_tk = &auth.split(' ').collect::<Vec<&str>>()[1];

        let grant_email = validate::decode_auth_token(auth_tk, Some("auth".into()))?.sub;

        let mut granter = Email::load_user(&grant_email)?;

        if !granter.is_empty() {
            Ok(granter.pop().unwrap())
        } else {
            Err(ResError::new(
                "Invalid token. Problem finding user".into(),
                401,
            ))
        }
    }
    /// Extracts the bearer authorization header
    fn extract_auth_header(req: &HttpRequest) -> Result<String, actix_web::error::ParseError> {
        let auth_header = Authorization::<Bearer>::parse(req)?;
        Ok(auth_header.into_scheme().to_string())
    }
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

        match key {
            "email" => Email::load_user(&identity.as_ref().unwrap()),
            _ => users
                .filter(username.eq(identity.as_ref().unwrap()))
                .load::<User>(&connect_to_db()),
        }
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

/// User Profile data from google Oauth
#[derive(Debug, Deserialize, Serialize)]
pub struct GoogleUser {
    /// Full name
    name: String,

    /// First name
    given_name: String,

    family_name: String,
    pub email: String,

    /// Verified_social_email
    verified_email: bool,

    /// Social ID
    id: String,

    /// avatar
    picture: String,
    locale: String,
}

/// Service Oauth User Object
/// Holds user social-authenticated user data
#[derive(Queryable, Serialize, AsChangeset, Deserialize, Identifiable)]
#[table_name = "oath_users"]
pub struct OauthGgUser {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub first_name: Option<String>,
    pub family_name: Option<String>,
    pub is_verified: bool,

    picture: Option<String>,
    locale: Option<String>,
    acc_id: String,
    pub is_active: bool,

    /// Oauth Account provider
    provider: String,

    /// Verified Oauth account used
    provider_verified: bool,

    #[serde(deserialize_with = "from_timestamp")]
    pub created_at: NaiveDateTime,
    #[serde(deserialize_with = "from_timestamp")]
    updated_at: NaiveDateTime,
}

impl OauthGgUser {
    /// Registers a user account using Oauth
    /// from a third party account
    ///
    /// # Arguments
    ///  `usr_data`: GoogleUser data holding the user account profile info
    ///
    ///  # Retuns
    ///  - `None` if account id exists
    ///  - `OauthGgUser`: Newly registered account data
    pub fn register_as_third_party(
        usr_data: &GoogleUser,
    ) -> Result<Option<(OauthGgUser, User)>, Box<dyn stdError>> {
        use rand::{distributions::Alphanumeric, thread_rng, Rng};

        use crate::diesel_cfg::schema::avatars::dsl::url as av_url;
        use crate::diesel_cfg::schema::emails::{active, email as e_email, user_id};
        use crate::diesel_cfg::schema::oath_users::dsl::*;
        use crate::diesel_cfg::schema::users::dsl::{
            social_id as usocial_id, username as u_username, users,
        };

        let present_user = users
            .find(Email::u_id(&usr_data.email)?)
            .select(usocial_id)
            .get_results::<Option<String>>(&connect_to_db())?;

        if present_user.is_empty() {
            // New User

            let acc_provider = "google";
            let new_data = (
                email.eq(&usr_data.email),
                name.eq(&usr_data.name),
                first_name.eq(&usr_data.given_name),
                family_name.eq(&usr_data.family_name),
                provider_verified.eq(&usr_data.verified_email),
                picture.eq(&usr_data.picture),
                locale.eq(&usr_data.locale),
                acc_id.eq(&usr_data.id),
                provider.eq(acc_provider),
            );
            let user = diesel::insert_into(oath_users)
                .values(&new_data)
                .get_result::<OauthGgUser>(&connect_to_db())?;

            // Insert to Users

            let _rnd_ext = thread_rng()
                .sample_iter(Alphanumeric)
                .take(10)
                .collect::<String>();
            let user_name = format!("{}-{}-{}", &usr_data.name, _rnd_ext, acc_provider);

            let ord_user = diesel::insert_into(users)
                .values(&(u_username.eq(user_name), usocial_id.eq(&usr_data.id)))
                .get_result::<User>(&connect_to_db())?;

            // Save active email

            let email_data = (
                e_email.eq(&usr_data.email),
                user_id.eq(&ord_user.id),
                active.eq(true),
            );
            diesel::insert_into(emails_table::table)
                .values(&email_data)
                .load::<Email>(&connect_to_db())?;

            NewProfile::new(ord_user.id, None)?;

            let avatar = Avatar::belonging_to(&ord_user).get_result::<Avatar>(&connect_to_db())?;
            diesel::update(&avatar)
                .set(av_url.eq(&user.picture))
                .get_result::<Avatar>(&connect_to_db())?;

            return Ok(Some((user, ord_user)));
        }

        let s_id = &present_user[0];
        match s_id {
            // Previously used Oauth account
            Some(s) => {
                diesel::update(oath_users.filter(acc_id.eq(s)))
                    .set((
                        picture.eq(&usr_data.picture),
                        name.eq(&usr_data.name),
                        first_name.eq(&usr_data.given_name),
                        family_name.eq(&usr_data.family_name),
                    ))
                    .execute(&connect_to_db())?;

                Ok(None)
            }
            // Existing user email
            None => Err("You seem to have an account with this email. Try signing in".into()),
        }
    }
}

/// Deletes the avatar file matching the given ID
fn remove_old_url(pub_id: &str) -> Result<String, ()> {
    remove_py_mod(pub_id)
}

impl Reftoken {
    /// Verifies a given refresh token in exchange for
    /// new auth and refresh tokens for the user
    ///
    /// # Arguments
    ///  given_tk: The refresh token to be verified
    pub async fn exchange_token(given_tk: &str) -> Result<Tokens, Error> {
        use crate::diesel_cfg::schema::refresh_tokens::dsl::*;

        // let t_hash = hash(given_tk, DEFAULT_COST).map_err(ErrorInternalServerError)?;
        let t_hash = given_tk;
        let token = refresh_tokens
            .filter(body.eq(t_hash))
            .load::<Reftoken>(&connect_to_db())
            .map_err(ErrorInternalServerError)?;
        if token.is_empty() || !&token[0].valid {
            return Err(ErrorForbidden("Invalid Token".to_string()));
        };

        let verified_tk = match validate::decode_auth_token(given_tk, Some("refresh".into())) {
            Ok(t) => t,
            Err(e) => return Err(ErrorForbidden(e.to_string())),
        };

        let (new_autht, new_ref_t) = Reftoken::generate_tokens(&verified_tk.sub)?;

        diesel::delete(&token[0])
            .execute(&connect_to_db())
            .map_err(|e| {
                debug!("{}", e);
                ErrorInternalServerError::<String>(e.to_string())
            })?;

        let mut new_rf_stct = NewRfToken {
            body: Cow::Borrowed(&new_ref_t),
        };
        new_rf_stct.save().await.map_err(ErrorInternalServerError)?;

        Ok((new_autht, new_ref_t))
    }

    /// Generated auth and refresh tokens
    ///  # Arguments
    ///  sub_field: sub encoding field
    pub fn generate_tokens(sub: &str) -> Result<(String, String), Error> {
        let auth_tk_duration = env::var("AUTH_TOKEN_DURATION")
            .unwrap_or_else(|e| {
                debug!("{}", e);
                "120".into()
            })
            .parse::<i64>()
            .map_err(|e| ErrorInternalServerError(e.to_string()))?;
        let auth_token = User::create_token(sub, Some(auth_tk_duration), "auth".into())
            .map_err(|e| ErrorInternalServerError(e.to_string()))?;

        let rf_duration = env::var("REFRESH_TOKEN_DURATION")
            .unwrap_or_else(|e| {
                debug!("{}", e);
                "42600".into()
            })
            .parse::<i64>()
            .map_err(|e| ErrorInternalServerError(e.to_string()))?;

        let refresh_tkn = User::create_token(sub, Some(rf_duration), "refresh".into())
            .map_err(|e| ErrorInternalServerError(e.to_string()))?;
        Ok((auth_token, refresh_tkn))
    }

    /// __ Marks a refresh token as invalid__
    /// Deletes the tokens from the associated table
    pub fn invalidate(token: &str) -> Result<usize, Error> {
        use crate::diesel_cfg::schema::refresh_tokens::dsl::*;

        /*
            hash(token, DEFAULT_COST)
                .map(|t_hash| t_hash)
                .map_err(ErrorInternalServerError)
                .and_then(|t_hash| {

            // diesel::update(refresh_tokens.filter(body.eq(token)))
            //     .set(valid.eq(false))
            //    .get_result::<Reftoken>(&connect_to_db())
            diesel::delete(refresh_tokens.filter(body.eq(t_hash))).execute(&connect_to_db())
                .map_err(ErrorInternalServerError)
        })*/
        diesel::delete(refresh_tokens.filter(body.eq(token)))
            .execute(&connect_to_db())
            .map_err(ErrorInternalServerError)
    }
}

impl<'a> NewRfToken<'a> {
    /// Saves a new refresh token to the refresh tokens table
    pub async fn save(&mut self) -> Result<(), String> {
        /* match hash(self.body.to_mut(), DEFAULT_COST) {
            Ok(t) => self.body = Cow::Owned(t),
            Err(e) => return Err(e.to_string())
        }*/

        if let Err(e) = diesel::insert_into(refresh_tokens::table)
            .values(&*self)
            .execute(&connect_to_db())
        {
            Err(e.to_string())
        } else {
            Ok(())
        }
    }
}
