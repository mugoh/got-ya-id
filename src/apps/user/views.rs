//! Handles views for User items
//!

use super::models::{NewUser, PassResetData, ResetPassData, SignInUser, User};
use super::utils::{get_context, get_reset_context};

use crate::apps::auth::validate;
use crate::core::mail;
use crate::core::response;

use lazy_static;
use log::{debug, error};
use url::Url;

use actix_web::{http, web, HttpResponse};
use serde_json::json;
use tera::{self, Context, Tera};
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
pub fn register_user(mut data: web::Json<NewUser>) -> HttpResponse {
    let user_ = data.0.clone();
    let token = validate::encode_jwt_token(user_).unwrap();
    let _claims = validate::decode_auth_token(&token);

    // -> Extract host info from req Headers
    let path = format!(r#"http://localhost:8888/api/auth/verify/{}"#, &token);
    let path = Url::parse(&path).unwrap().to_string();

    if let Err(err) = data.validate() {
        let res: response::JsonErrResponse<_> =
            response::JsonErrResponse::new(http::StatusCode::BAD_REQUEST.to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
        // Filter json where message is not null
    };

    match data.save() {
        Ok(saved_user) => saved_user,
        Err(e) => {
            let res: response::JsonErrResponse<_> =
                response::JsonErrResponse::new("409".to_string(), e);
            return HttpResponse::build(http::StatusCode::CONFLICT).json(&res);
        }
    };

    // Mail
    let context: Context = get_context(&data.0, &path);
    match TEMPLATE.render("email_activation.html", &context) {
        Ok(s) => {
            let mut mail = mail::Mail::new(
                &data.0.email.to_mut(),
                &data.0.username.to_mut(),
                "Email activation".to_string(),
                &s,
            );
            mail.send().unwrap();
        }

        Err(e) => {
            for er in e.iter().skip(1) {
                error!("Reason: {}", er);
            }
        }
    };

    let res: response::JsonResponse<_> = response::JsonResponse::new(
        http::StatusCode::CREATED.to_string(),
        format!(
            "Success. An activation link sent to {}",
            &data.0.email.clone()
        ),
        json!({"email": &data.0.email, "username": &data.0.username, "token": &token}),
    );

    HttpResponse::build(http::StatusCode::CREATED).json(&res)
}

/// Logs in registered user
///
/// # method: POST
///
pub fn login(user: web::Json<SignInUser>) -> HttpResponse {
    if let Err(err) = user.validate() {
        let res = response::JsonErrResponse::new("400".to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
    };
    if user.has_credentials() {
        let res =
            response::JsonErrResponse::new("400".to_string(), "Oh-uh, provide a username or email");
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
    }
    let res = match user.sign_in() {
        Ok(usr_vec) => {
            if usr_vec.is_empty() {
                return HttpResponse::build(http::StatusCode::UNAUTHORIZED).json(
                    response::JsonErrResponse::new(
                        http::StatusCode::UNAUTHORIZED.to_string(),
                        "Could not find details that match you. Just try again.",
                    ),
                );
            }
            let usr = &usr_vec[0];
            if !usr.verify_pass(user.get_password()).unwrap() {
                let status = http::StatusCode::UNAUTHORIZED;
                return HttpResponse::build(status).json(response::JsonErrResponse::new(
                    status.to_string(),
                    "Could not find details that match you. Just try again.",
                ));
            }
            match usr.create_token(&usr.email) {
                Ok(s) => response::JsonResponse::new(
                    http::StatusCode::OK.to_string(),
                    "Login Success".to_string(),
                    json!(
                        { "username": &usr.username,
                          "token": &s
                        }
                    ),
                ),
                Err(e) => {
                    debug!("{:?}", e);
                    let status = http::StatusCode::INTERNAL_SERVER_ERROR;
                    let e = response::JsonErrResponse::new(
                        status.to_string(),
                        "Encountered a problem attempting to sign in. Try again later".to_string(),
                    );
                    return HttpResponse::build(status).json(e);
                }
            }
        }
        Err(e) => {
            return HttpResponse::build(http::StatusCode::UNAUTHORIZED).json(
                response::JsonErrResponse::new(
                    http::StatusCode::UNAUTHORIZED.to_string(),
                    format!(
                        "Could not find details that match you. Just try again. : {}",
                        e
                    ),
                ),
            );
        }
    };

    HttpResponse::build(http::StatusCode::OK).json(res)
}

/// Verifies a user's account.
/// The user is retrived from the token passed in the URL Path
pub fn verify(path: web::Path<String>) -> HttpResponse {
    match User::verify_user(&path) {
        Ok(user) => {
            let res = response::JsonResponse::new(
                http::StatusCode::OK.to_string(),
                format!("Success. Account of user {} verified", user.email),
                json!({
                    "username": &user.username,
                    "email": &user.email,
                    "is_verified": &user.is_verified
                }),
            );
            return HttpResponse::build(http::StatusCode::OK).json(&res);
        }
        Err(e) => {
            let res = response::JsonErrResponse::new(
                http::StatusCode::FORBIDDEN.to_string(),
                format!("Account verification failed: {}", e),
            );
            return HttpResponse::build(http::StatusCode::FORBIDDEN).json(&res);
        }
    };
}

/// Sends a Password Reset Email
///
/// # method
/// ## POST
pub fn send_reset_email(mut data: web::Json<PassResetData>) -> HttpResponse {
    if let Err(err) = data.validate() {
        let res = response::JsonErrResponse::new("400".to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
    };
    let user = match User::find_by_email(&data.email.to_mut()) {
        Ok(usr) => usr,
        Err(e) => {
            let status = http::StatusCode::NOT_FOUND;
            return HttpResponse::build(status)
                .json(err_response(status.to_string(), format!("{}", e)));
        }
    };
    let user = &user[0];
    let token = user.create_token(&user.email).unwrap();
    let path = format!("http://api/auth/{}", token);
    let context: Context = get_reset_context(&user, &path);
    match TEMPLATE.render("password_reset.html", &context) {
        Ok(s) => {
            let mut mail = mail::Mail::new(
                &user.email,
                &user.username,
                "Account password reset".to_string(),
                &s,
            );
            mail.send().unwrap();
        }

        Err(e) => {
            for er in e.iter().skip(1) {
                error!("Reason: {}", er);
            }
        }
    };
    let res = response::JsonResponse::new(
        http::StatusCode::OK.to_string(),
        format!("Success. A password reset link sent to {}", &user.email),
        json!({"email": &user.email, "username": &user.username, "link": &path, "token": token}),
    );

    HttpResponse::build(http::StatusCode::OK).json(&res)
}

/// Allows reset of user account passwords
///
/// # Method
/// ## PATCH
///
/// # url
/// `auth/password/reset/{token}`
///
pub fn reset_password(data: web::Json<ResetPassData>, path: web::Path<String>) -> HttpResponse {
    if let Err(err) = data.validate() {
        let res = response::JsonErrResponse::new("400".to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
    };

    match User::reset_pass(&path, &data.password) {
        Ok(_) => {
            let res = response::JsonResponse::new(
                http::StatusCode::OK.to_string(),
                "Success. Account password changed".to_string(),
                "",
            );

            HttpResponse::build(http::StatusCode::OK).json(&res)
        }
        Err(e) => {
            let status = http::StatusCode::UNAUTHORIZED;
            HttpResponse::build(status).json(err_response(
                status.to_string(),
                format!("Failed to reset password: {:?}", e),
            ))
        }
    }
}

lazy_static! {
    /// Lazily Compiles Templates
    static ref TEMPLATE: Tera = {
        let mut tera = tera::compile_templates!("src/templates/*");
        tera.autoescape_on(vec![".sql"]);
        tera
    };
}

/// Gives Err Json Response
fn err_response<T>(status: String, msg: T) -> response::JsonErrResponse<T> {
    response::JsonErrResponse::new(status, msg)
}
