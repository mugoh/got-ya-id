//! Handles views for User items
//!

use crate::apps::auth::validate;
use crate::apps::user::models::User;
use actix_web::{http, web, HttpResponse};

use validator::Validate;

/// Registers a new user
///
/// # method
///
///
/// # Returns
/// JSON of received User data
pub fn register_user(data: web::Json<User>) -> HttpResponse {
    // jwt
    let user_ = data.0.clone();
    let token = validate::encode_jwt_token(user_).unwrap();
    let _claims = validate::decode_auth_token(&token);

    if let Err(err) = data.validate() {
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(err);
        // Filter json where message is not null
    };
    HttpResponse::build(http::StatusCode::CREATED).json(data.0)
}
