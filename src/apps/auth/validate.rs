//! User authentication

use crate::apps::user::models::User;
use crate::config::config;
use jsonwebtoken as jwt;
use jwt::{decode, encode, Header, Validation};
use serde_derive::{Deserialize, Serialize};
use std::{error, process};

/// JWT Auth Identity
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub company: String,
    pub exp: usize,
}

/// Encodes a JWT token with user details {email, username}
pub fn encode_jwt_token(user: User) -> Result<String, Box<dyn error::Error>> {
    let payload = Claims {
        sub: user.email.unwrap().to_owned(),
        username: user.username.unwrap().to_owned(),
        company: "ACME".to_owned(),
        exp: 10000000000,
    };

    // ENV Configuration
    let conf = config::get_env_config().unwrap_or_else(|err| {
        eprintln!("Error: Missing required ENV Variable\n{:#?}", err);
        process::exit(78);
    });
    let key = &conf.secret_key;

    let mut header = Header::default();
    header.kid = Some("secretssec".to_owned());

    match encode(&header, &payload, key.as_ref()) {
        Ok(t) => Ok(t),
        Err(e) => Result::Err(Box::new(e)), // println!("{:?}", e),
    }
}

/// Decodes an encoded authorization token
///
/// # Returns
/// ---------
/// Token Claims
///
/// struct Claims {
///    pub sub: String,
///    pub username: String,
///    pub company: String,
///    pub exp: usize,
/// }
///
/// # Panics
/// - If the token decoding fails
///
pub fn decode_auth_token(token: &String) -> Result<Claims, Box<dyn error::Error>> {
    // ENV Configuration
    let conf = config::get_env_config().unwrap_or_else(|err| {
        eprintln!("Error: Missing required ENV Variable\n{:#?}", err);
        process::exit(78);
    });
    let key = &conf.secret_key;

    let decoded_token = match decode::<Claims>(&token, key.as_ref(), &Validation::default()) {
        Ok(c) => c,
        Err(e) => return Result::Err(Box::new(e)), // println!("{:?}", e),
    };
    Ok(decoded_token.claims)
}
