//! Handles views for User items
//!

use crate::apps::auth::validate;
use crate::apps::user::models::User;
use crate::core::mail;
use crate::core::response;

use lazy_static;
use log::debug;
use url::Url;

use actix_web::{http, web, HttpResponse};
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
pub fn register_user(data: web::Json<User>) -> HttpResponse {
    let user_ = data.0.clone();
    let token = validate::encode_jwt_token(user_).unwrap();
    let _claims = validate::decode_auth_token(&token);
    let path = format!(r"http://{:?}", &token);
    let path = Url::parse(&path).unwrap();

    // Mail
    let context: Context = get_context(&data.0, &path.to_string());
    match TEMPLATE.render("email_activation.html", &context) {
        Ok(s) => {
            println!("{}", s);
            let mut mail = mail::Mail::new(
                &data.0.email.clone().unwrap(),
                &data.0.username.clone().unwrap(),
                "Email activation".to_string(),
                &s,
            );
            mail.send().unwrap();
        }

        Err(e) => {
            for er in e.iter().skip(1) {
                debug!("Reason: {}", er);
            }
        }
    };

    if let Err(err) = data.validate() {
        let res: response::JsonErrResponse<_> =
            response::JsonErrResponse::new(http::StatusCode::BAD_REQUEST.to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
        // Filter json where message is not null
    };
    let res: response::JsonResponse<_> = response::JsonResponse::new(
        http::StatusCode::CREATED.to_string(),
        format!(
            "Success. An activation link sent to {}",
            &data.0.email.clone().unwrap()
        ),
        data.0.clone(),
    );

    HttpResponse::build(http::StatusCode::CREATED).json(&res)
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
fn get_context(data: &User, path: &String) -> Context {
    let mut context = Context::new();

    context.insert("username", &data.username);
    context.insert("link", path);
    context
}
