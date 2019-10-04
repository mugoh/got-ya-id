//! Handles views for User items
//!

use crate::apps::auth::validate;
use crate::apps::user::models::{NewUser, SignInUser};
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
    let path = format!(r"http://{:?}", &token);
    let path = Url::parse(&path).unwrap();

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
    let context: Context = get_context(&data.0, &path.to_string());
    match TEMPLATE.render("email_activation.html", &context) {
        Ok(s) => {
            let mut mail = mail::Mail::new(
                &data.0.email.clone(),
                &data.0.username.clone(),
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
        json!({"email": &data.0.email, "username": &data.0.username}),
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
        Err(_) => {
            return HttpResponse::build(http::StatusCode::UNAUTHORIZED).json(
                response::JsonErrResponse::new(
                    http::StatusCode::UNAUTHORIZED.to_string(),
                    "Could not find details that match you. Just try again.",
                ),
            );
        }
    };

    HttpResponse::build(http::StatusCode::OK).json(res)
}

lazy_static! {
    /// Lazily Compiles Templates
    static ref TEMPLATE: Tera = {
        let mut tera = tera::compile_templates!("src/templates/*");
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

/// Returns the context holding the template variables
///
/// # Returns
/// - tera::Context
fn get_context(data: &NewUser, path: &String) -> Context {
    let mut context = Context::new();

    context.insert("username", &data.username);
    context.insert("link", path);
    context
}
