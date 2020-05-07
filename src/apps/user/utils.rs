use regex::Regex;
use tera::{self, Context, Tera};
use validator::ValidationError;

use super::models::User;

use crate::apps::core::response;

use chrono::NaiveDateTime;
use serde::de;
use std::fmt;

lazy_static! {

    /// Lazily Compiled Templates
    pub static ref TEMPLATE: Tera = {
    Tera::new("src/templates/**/*").unwrap()

    };
}
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
        static ref NAME_PATTERN: Regex = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
    }
    if !NAME_PATTERN.is_match(name) {
        return Err(ValidationError::new("Name should only contain letters"));
    }
    Ok(())
}

/// Validates Email
/// - Ensures the email input follows a valid email
/// address format
///
///
///  # Returns
///
///  ## ValidationError
/// If the validation fails
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    lazy_static! {
        static ref EMAIL_PATTERN: Regex =
            Regex::new(r"(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)").unwrap();
    }
    if !EMAIL_PATTERN.is_match(email) {
        return Err(ValidationError::new("Oops! Email format not invented"));
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

/// Returns the context holding the template variables
///
/// This is the email activation context
///
/// # Arguments
/// `username`: Greeting name
/// `path`: The activation link
///
/// # Returns
/// - tera::Context
pub fn get_context(username: Option<&str>, path: &str) -> Context {
    let mut context = Context::new();

    if let Some(name) = username {
        context.insert("username", name);
    }

    context.insert("link", path);
    context
}

/// Template holding context for password reset
/// Receives a User ref
pub fn get_reset_context<'a>(data: &User, path: &'a str) -> Context {
    let mut context = Context::new();

    context.insert("username", &data.username);
    context.insert("link", &path);
    context
}

/// NaiveDateTime Serialize Deserialize implementation
pub mod naive_date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f %:z";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

/// Creates a Err Json Response
/// using the given arguments
///
pub fn err_response<T>(status: String, msg: T) -> response::JsonErrResponse<T> {
    response::JsonErrResponse::new(status, msg)
}

/// Builds a complete URI from the arguments given
///
/// # Arguments
/// ## host: str
/// The host part of the URL
///
/// ## path: str
/// Path of the request
///
/// ## id: str
/// Parameter to append to complete the url path
pub fn get_url<'a>(host: &'a str, path: &'a str, id: &'a str) -> String {
    format!(
        r#"http://{host}/{path}/{id}"#,
        host = host,
        path = path,
        id = id
    )
    .replace("\"", "")
    // HeaderValue can't be formatted to str
}

/// Creates an oauth2 BasicClient for use in google-auth
/// authentication
pub fn create_oauth_client() -> oauth2::basic::BasicClient {
    use oauth2::basic::BasicClient;
    use oauth2::prelude::*;
    use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, Scope, TokenUrl};

    use url::Url;

    let google_client_id = ClientId::new(
        std::env::var("GOOGLE_CLIENT_ID")
            .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new(
        Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .expect("Invalid authorization endpoint URL"),
    );
    let token_url = TokenUrl::new(
        Url::parse("https://www.googleapis.com/oauth2/v3/token")
            .expect("Invalid token endpoint URL"),
    );

    BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .add_scope(Scope::new("openid".to_string()))
    .add_scope(Scope::new("email".to_string()))
    .add_scope(Scope::new("profile".to_string()))
    .add_scope(Scope::new(
        "https://www.googleapis.com/auth/plus.me".to_string(),
    ))
    .set_redirect_url(RedirectUrl::new(
        // host.join("api/auth/callback")
        //     .expect("Invalid redirect Url"),
        Url::parse("http://127.0.0.1:8888/api/auth/callback").expect("Invalid RedirectUrl"),
    ))
}

struct NaiveDateTimeVisitor;

impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string represents chrono::NaiveDateTime")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%f %:z") {
            Ok(t) => Ok(t),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
        }
    }
}

pub fn from_timestamp<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(NaiveDateTimeVisitor)
}

struct NaiveYear;

impl<'de> de::Visitor<'de> for NaiveYear {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string represents chrono::NaiveDateTime")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match NaiveDateTime::parse_from_str(s, "%Y") {
            Ok(t) => Ok(t),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
        }
    }
}

pub fn to_year<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(NaiveDateTimeVisitor)
}
