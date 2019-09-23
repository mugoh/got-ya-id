//! Handles views for User items
//!

use crate::apps::auth::validate;
use crate::apps::core::response;
use crate::apps::user::models::User;

use actix_web::{http, web, HttpResponse};
use validator::Validate;

/// Registers a new user
///
/// # methods
/// - ## POST
///
/// # Returns
/// - On Sucess: JSONResponse
/// - On ERROR: JSONErrResponse
///
pub fn register_user(data: web::Json<User>) -> HttpResponse {
    let user_ = data.0.clone();
    let token = validate::encode_jwt_token(user_).unwrap();
    let _claims = validate::decode_auth_token(&token);

    if let Err(err) = data.validate() {
        let res = response::JsonErrResponse::new(http::StatusCode::BAD_REQUEST.to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
        // Filter json where message is not null
    };
    let res = response::JsonResponse::new(
        http::StatusCode::CREATED.to_string(),
        format!(
            "Success. An activation link sent to {}",
            &data.0.email.clone().unwrap()
        ),
        data.0.clone(),
    );

    HttpResponse::build(http::StatusCode::CREATED).json(&res)
}
