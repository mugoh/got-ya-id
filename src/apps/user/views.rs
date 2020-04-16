//! Handles views for User items
//!

use super::models::{
    NewUser, OClient, OauthInfo, PassResetData, ResetPassData, SignInUser, User, UserEmail,
};
use super::utils::{err_response, get_context, get_reset_context, get_url, TEMPLATE};

use crate::apps::auth::validate;
use crate::core::mail;
use crate::core::response::{self, err, respond};
use crate::hashmap;

use log::{debug, error};
use tera::{self, Context};

use actix_web::{http, web, HttpRequest, HttpResponse};
use serde_json::json;
use validator::Validate;

use std::sync::{Arc, Mutex};

use actix_web::http::header::Header;
use actix_web_httpauth::headers::authorization::Authorization;
use actix_web_httpauth::headers::authorization::Bearer;
// use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;

/// Registers a new user
///
/// # url
/// ## `auth`
/// /auth
///
/// # method
/// - ## POST
///
/// # Returns
/// - On Sucess: JSONResponse
/// - On ERROR: JSONErrResponse
///
pub fn register_user(mut data: web::Json<NewUser>, req: HttpRequest) -> HttpResponse {
    let user_ = data.0.clone();
    let token = validate::encode_jwt_token(user_).unwrap();
    let _claims = validate::decode_auth_token(&token);

    // -> Extract host info from req Headers
    let host = format!("{:?}", req.headers().get("host").unwrap());
    let path = get_url(&host, "api/auth/verify", &token);

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
    match TEMPLATE.render("email_activation.html", context) {
        Ok(s) => {
            let mut mail =
                mail::Mail::new(&data.0.email, &data.0.username, "Email activation", &s).unwrap();
            mail.send().unwrap();
        }

        Err(e) => {
            for er in e.iter() {
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
/// # url
/// ## `auth/login`
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

            if !usr.is_active {
                return HttpResponse::Forbidden().json(response::JsonErrResponse::new(
                    http::StatusCode::FORBIDDEN.to_string(),
                    &format!("Account associated with email {} is deactivated", usr.email),
                ));
            }
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
///
/// # url
/// ## `auth/verify/{token}`
///
/// # Method
/// ## GET
///
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
/// # url
/// ## `auth/password/request`
///
/// # Method
/// ## POST
pub fn send_reset_email(mut data: web::Json<PassResetData>, req: HttpRequest) -> HttpResponse {
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
    let host = format!("{:?}", req.headers().get("host").unwrap());
    let path = get_url(&host, "api/auth", &token);
    let context: Context = get_reset_context(&user, &path);
    match TEMPLATE.render("password_reset.html", context) {
        Ok(s) => {
            let mut mail = mail::Mail::new(
                &user.email,
                &user.username,
                "Account password reset",
                s.as_str(),
            )
            .unwrap();

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

    HttpResponse::Created().json(&res)
}

/// Allows reset of user account passwords
///
/// This is path accessible from the password reset link
/// sent to the registered user email
///
/// # url
/// ## `auth/password/reset/{token}`
///
/// # Method
/// ## PATCH
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

/// Retrieves a user and their profile by ID
///
/// # url
/// ## `/user/{ID}`
pub fn get_user(id: web::Path<i32>) -> HttpResponse {
    match User::find_by_pk(*id, Some(1)) {
        Ok((usr, profile)) => {
            let data = hashmap!["status" => "200", "message" => "Success. User and User profile retrieved"];
            println!("{:?}\n\n{:?}", usr, profile);
            respond(data, Some((usr, profile.unwrap())), None).unwrap()
        }
        Err(e) => err("404", e.to_string()),
    }
}

/// Activates or Deactivates User accounts
///
/// The activation status is updated to !current_activation_status
/// (Opposite bool of the current)
///
/// # url
/// ## `/auth/deactivate`
///
/// # method
///  PATCH
pub fn change_activation_status(mut data: web::Json<UserEmail>) -> HttpResponse {
    if let Err(err) = data.validate() {
        let res = response::JsonErrResponse::new("400".to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
    };
    match User::find_by_email(data.email.to_mut()) {
        Ok(vec) => {
            let user = &vec[0];
            match user.alter_activation_status() {
                Ok(usr) => {
                    let data =
                        hashmap!["status" => "200", "message" => "User activation status changed"];
                    let body = json!({
                        "email": usr.email,
                        "username": usr.username,
                        "is_active": usr.is_active
                    });

                    respond(data, Some(body), None).unwrap()
                }
                Err(e) => err("500", e.to_string()),
            }
        }
        Err(e) => err("404", e.to_string()),
    }
}

/// Oauth authentication
///
/// Authenticates user using google-auth. This endpoint returns
/// an authentication url which calls the callback endpoint
/// `/auth/callback` on success
///
///
/// # url
/// ## `/auth/google`
///
/// # method
///  GET
pub fn google_auth(_req: HttpRequest, data: web::Data<Arc<Mutex<OClient>>>) -> HttpResponse {
    use oauth2::CsrfToken;

    // TODO Retrieve base url for redirect url
    // let host = format!("http://{:?}", req.headers().get("host").unwrap());
    // let host = Url::parse(&host).unwrap();

    let client = &data.get_ref().lock().unwrap().client;
    let (auth_url, _csrf_token) = client.authorize_url(CsrfToken::new_random);

    let data = hashmap!["status" => "200", "message" => "Authentication success. Browse to the authentication url given"];
    let body = json!({ "auth_url": auth_url.to_string() });
    std::mem::drop(client);

    respond(data, Some(body), None).unwrap()
}

/// Oauth Url Callback
///
///
/// Exchanges the Oauth code for a user authenication token.
/// This is endpoint is called once the user agrees to grant access
/// to the app
///
/// # url
/// ## `/auth/callback`
///
/// # method
///  GET
pub fn google_auth_callback(
    info: web::Query<OauthInfo>,
    data: web::Data<Arc<Mutex<OClient>>>,
) -> HttpResponse {
    //

    use oauth2::prelude::*;
    use oauth2::AuthorizationCode;

    let client = &data.get_ref().lock().unwrap().client;
    let code = AuthorizationCode::new(info.code.to_string());

    let token_data = match client.exchange_code(code) {
        Ok(token) => {
            let data = hashmap!["status" => "200",
            "message" => "Success. Authorization token received"];

            respond(data, Some(token), None).unwrap()
        }
        Err(er) => err("403", er.to_string()),
    };

    std::mem::drop(client);
    token_data
}

/// Registers users authenticated with google Oauth
///
/// # url
/// `auth/register/social`
///
/// # method
/// get
///
/// # Arguments
/// `Authorization: Bearer`
pub fn register_g_oauth(req: HttpRequest) -> HttpResponse {
    let token_hdr = match Authorization::<Bearer>::parse(&req) {
        Ok(auth_header) => auth_header.into_scheme().to_owned().to_string(),
        Err(e) => return err("400", e.to_string()),
    };

    let token = &token_hdr.split(' ').collect::<Vec<&str>>()[1];
    HttpResponse::build(http::StatusCode::OK).json(token)
}
