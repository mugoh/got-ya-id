//! Handles views for User items

use super::{
    models::{
        GoogleUser, NewJsonUser, NewRfToken, NewUser, NewUserLevel, OClient, OauthGgUser,
        OauthInfo, Reftoken, ResetPassData, SignInUser, User, UserEmail,
    },
    utils::{err_response, get_context, get_reset_context, get_url, TEMPLATE},
};
use crate::{
    apps::auth::validate,
    core::{
        mail,
        response::{self, err, respond},
    },
    hashmap,
};

use tera::{self, Context};

use actix_web::{
    error::ErrorInternalServerError, http, web, Error, HttpRequest, HttpResponse, Result,
};
use serde_json::json;
use validator::Validate;

use std::{
    borrow::Cow,
    env,
    sync::{Arc, Mutex},
};

use actix_web::http::header::Header;
use actix_web_httpauth::headers::authorization::Authorization;
use actix_web_httpauth::headers::authorization::Bearer;

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
pub async fn register_user(
    data: web::Json<NewJsonUser<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user_ = &data.0;
    let token =
        validate::encode_jwt_token(user_.email.as_ref().into(), "verification".into()).unwrap();

    // -> Extract host info from req Headers
    // let host = format!("{:?}", req.headers().get("host").unwrap());
    let req_header = req.headers().get("host");
    let host = if let Some(rq) = req_header {
        format!("{:?}", rq)
    } else {
        "http::/127.0.0.1:8888".into()
    };
    let path = get_url(&host, "api/auth/verify", &token);

    if let Err(err) = data.validate() {
        let res: response::JsonErrResponse<_> =
            response::JsonErrResponse::new(http::StatusCode::BAD_REQUEST.to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST)
            .json(&res)
            .await;
        // Filter json where message is not null
    };

    let mut user = user_.into_savable();

    match user.save(&data.0.email) {
        Ok(saved_user) => saved_user,
        Err(e) => {
            let res: response::JsonErrResponse<_> =
                response::JsonErrResponse::new("409".to_string(), e.to_string());
            return HttpResponse::build(http::StatusCode::CONFLICT)
                .json(&res)
                .await;
        }
    };

    // Mail
    send_activation_link(
        &data.email,
        Some(&data.username),
        &path,
        "email_activation.html",
    )
    .await?;
    let res: response::JsonResponse<_> = response::JsonResponse::new(
        http::StatusCode::CREATED.to_string(),
        format!("sucess. An activation link sent to {}", &data.0.email),
        json!({"email": &data.0.email, "username": &data.0.username, "token": &token}),
    );

    HttpResponse::build(http::StatusCode::CREATED)
        .json(&res)
        .await
}

/// Sends an account activation link to a user email
///
/// This endpoint should specifically be useful in re-sending
/// of account activation links to users. (Logic assumes, in cases
/// where the initial link token expired before use)
///
/// # url
/// `/auth/activation/send`
///
/// # method
///
/// `POST`
///
/// ### Authentication Required
pub async fn send_account_activation_link(
    email: web::Json<UserEmail<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    User::decode_auth_header(&req)?;

    if let Err(e) = email.0.validate() {
        return HttpResponse::build(http::StatusCode::BAD_REQUEST)
            .json(e)
            .await;
    }
    if let Err(e) = User::find_by_email(&email.email) {
        return HttpResponse::build(http::StatusCode::NOT_FOUND)
            .json(e)
            .await;
    }

    let token = User::create_token(&email.email, Some(24 * 60), "verification".into()).unwrap();
    let host = format!("{:?}", req.headers().get("host").unwrap());
    let path = get_url(&host, "api/auth/verify", &token);

    send_activation_link(&email.email, None, &path, "email_activation.html").await?;

    let data = hashmap!["status" => "200", "message" => "sucess. Activation link sent"];
    Ok(respond(data, Some("".to_string()), None).unwrap())
}

/// Logs in registered user
///
/// # method: POST
///
/// # url
/// ## `auth/login`
pub async fn login(user: web::Json<SignInUser<'_>>) -> Result<HttpResponse, Error> {
    if let Err(err) = user.validate() {
        let res = response::JsonErrResponse::new("400".to_string(), err);
        return Ok(HttpResponse::build(http::StatusCode::BAD_REQUEST)
            .json(&res)
            .await?);
    };
    if user.has_credentials() {
        let res =
            response::JsonErrResponse::new("400".to_string(), "Oh-uh, provide a username or email");
        return Ok(HttpResponse::build(http::StatusCode::BAD_REQUEST)
            .json(&res)
            .await?);
    }

    let mut reactication_msg = "";

    let res = match user.sign_in() {
        Ok(usr_vec) => {
            if usr_vec.is_empty() {
                let resp = HttpResponse::build(http::StatusCode::UNAUTHORIZED).json(
                    response::JsonErrResponse::new(
                        http::StatusCode::UNAUTHORIZED.to_string(),
                        "Could not find details that match you. Just try again.",
                    ),
                );
                return Ok(resp);
            }
            let usr = &usr_vec[0];

            if !usr.is_active {
                if let Err(e) = usr.alter_activation_status() {
                    debug!("{:?}", e);
                    return Ok(HttpResponse::InternalServerError()
                        .json(response::JsonErrResponse::new(
                            http::StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                            "Encountered a problem reacticating the account",
                        ))
                        .await?);
                } else {
                    reactication_msg = "Account activated. ";
                }
                /*
                    return HttpResponse::Forbidden().json(response::JsonErrResponse::new(
                        http::StatusCode::FORBIDDEN.to_string(),
                        &format!("Account associated with email {} is deactivated", usr.email),
                    ));
                */
            }
            if !usr
                .verify_pass(user.get_password())
                .await
                .map_err(ErrorInternalServerError)?
            {
                let status = http::StatusCode::UNAUTHORIZED;
                return Ok(HttpResponse::build(status)
                    .json(response::JsonErrResponse::new(
                        status.to_string(),
                        "Could not find details that match you. Just try again.",
                    ))
                    .await?);
            }
            let (auth_token, refresh_tkn) = generate_tokens(usr).await?;
            let mut rf_struct = NewRfToken {
                body: Cow::Borrowed(&refresh_tkn),
            };
            rf_struct.save().await.map_err(ErrorInternalServerError)?;

            response::JsonResponse::new(
                http::StatusCode::OK.to_string(),
                format!("{}Login success", reactication_msg),
                json!(
                    { "username": &usr.username,
                      "email": &usr.email(),
                      "auth_token": &auth_token,
                      "refresh_token": &refresh_tkn,
                    }
                ),
            )
        }
        Err(e) => {
            return Ok(HttpResponse::build(http::StatusCode::UNAUTHORIZED)
                .json(response::JsonErrResponse::new(
                    http::StatusCode::UNAUTHORIZED.to_string(),
                    format!(
                        "Could not find details that match you. Just try again. : {}",
                        e
                    ),
                ))
                .await?);
        }
    };

    Ok(HttpResponse::build(http::StatusCode::OK).json(res).await?)
}

/// Verifies a user's account.
/// The user is retrived from the token passed in the URL Path
///
/// # url
/// ## `auth/verify/{token}`
///
/// # Method
/// ## GET
pub fn verify(path: web::Path<String>) -> HttpResponse {
    match User::verify_user(&path) {
        Ok(_) => HttpResponse::build(http::StatusCode::OK).body("Yay! Your account is now verified"),
        Err(_) => HttpResponse::build(http::StatusCode::FORBIDDEN)
            .body("Oopsy! The link you used seems expired. Just request a resend of the account activation link"),
    }
}

/// Exchanges a refresh token for a new user authorization token
///
/// # url
/// `auth/refresh/{refresh}`
///
/// # Method
/// `GET`
pub async fn refresh_access_token(ref_tkn: web::Path<String>) -> Result<HttpResponse, Error> {
    let tokens = Reftoken::exchange_token(&ref_tkn.into_inner()).await?;

    let msg = hashmap![
            "status" => "200",
            "message" => "success. tokens updated"];

    let data = json!({
        "auth_token": tokens.0,
        "refresh_tokens": tokens.1
    });

    respond(msg, Some(data), None)?.await
}

/// Logs out authenticated users
/// The refresh token is invalidated
///
/// # url
/// `auth/logout`
///
/// # method
/// `GET`
pub async fn logout(ref_tkn: web::Path<String>) -> Result<HttpResponse, Error> {
    let res = Reftoken::invalidate(&ref_tkn.into_inner())?;
    if res == 0 {
        err("401", "Invalid token".to_string()).await
    } else {
        let msg = hashmap![
            "status" => "200",
            "message" => "success. tokens revoked"];
        respond(msg, Some("".to_string()), None)?.await
    }
}

/// Sends a Password Reset Email
///
/// # url
/// ## `auth/password/request`
///
/// # Method
/// ## POST
pub async fn send_reset_email(
    mut data: web::Json<UserEmail<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if let Err(err) = data.validate() {
        let res = response::JsonErrResponse::new("400".to_string(), err);
        return HttpResponse::build(http::StatusCode::BAD_REQUEST)
            .json(&res)
            .await;
    };

    let user = match User::find_by_email(&data.email.to_mut()) {
        Ok(usr) => usr,
        Err(e) => {
            let status = http::StatusCode::NOT_FOUND;
            return HttpResponse::build(status)
                .json(err_response(status.to_string(), e))
                .await;
        }
    };
    let user = &user[0];
    let token = User::create_token(&user.email, Some(59), "password_reset".into()).unwrap();
    let host = format!("{:?}", req.headers().get("host").unwrap());
    let path = get_url(&host, "api/auth", &token);
    let context: Context = get_reset_context(&user, &path);
    match TEMPLATE.render("password_reset.html", &context) {
        Ok(s) => {
            let mut mail = mail::Mail::new(
                &user.email,
                &user.username,
                "Account password reset",
                s.as_str(),
            )
            .await
            .map_err(ErrorInternalServerError)?;

            mail.send().await.map_err(ErrorInternalServerError)?;
        }

        Err(e) => return err("500", e.to_string()).await,
    };
    let res = response::JsonResponse::new(
        http::StatusCode::OK.to_string(),
        format!("sucess. A password reset link sent to {}", &user.email),
        json!({"email": &user.email, "username": &user.username, "link": &path, "token": token}),
    );

    HttpResponse::Created().json(&res).await
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
/// ## GET
///
pub fn reset_password(
    tmpl: web::Data<tera::Tera>,
    query: web::Query<std::collections::HashMap<String, String>>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let data: ResetPassData;
    let host = format!("{:?}", req.headers().get("host").unwrap());

    let host = format!(
        r#"http://{host}/{path}/{id}"#,
        host = host,
        path = "api/auth/password/reset",
        id = path
    )
    .replace("\"", "");

    // submitted form
    let mut ctx = tera::Context::new();
    ctx.insert("link", &host.as_str());

    if let Some(name) = query.get("new password") {
        ctx.insert("name", &name.to_owned());
        data = ResetPassData {
            password: query.get("new password").unwrap().to_owned(),
            password_conf: query.get("confirm password").unwrap().to_owned(),
        };
        if let Err(err) = data.validate() {
            let res = response::JsonErrResponse::new("400".to_string(), err);
            return HttpResponse::build(http::StatusCode::BAD_REQUEST).json(&res);
        };
    } else {
        let s = tmpl.render("password_reset_form.html", &ctx).unwrap();
        return HttpResponse::build(http::StatusCode::OK)
            .content_type("text/html")
            .body(s);
    }

    let s = tmpl.render("password_reset_form.html", &ctx).unwrap();
    match User::reset_pass(&path, &data.password) {
        Ok(_) => HttpResponse::build(http::StatusCode::OK)
            .content_type("text/html")
            .body(s),
        Err(e) => {
            let status = http::StatusCode::UNAUTHORIZED;
            HttpResponse::build(status).body(format!(
                "There was a problem resetting your password: {:?}.\n Request a resend of a new password reset link",
                e
            ))
        }
    }
}

/// Retrieves a user and their profile by ID
///
/// # url
/// ## `/user/{ID}`
pub async fn get_user(id: web::Path<i32>, req: HttpRequest) -> Result<HttpResponse, Error> {
    match User::find_by_pk_authenticated(*id, Some(1), &req) {
        Ok((usr, profile)) => {
            let data =
                hashmap!["status" => "200", "message" => "sucess. User and User profile retrieved"];
            respond(data, Some((usr, profile.unwrap())), None)
                .unwrap()
                .await
        }
        Err(e) => Err(e.into()),
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
///
/// ### Authentication Required
pub async fn change_activation_status(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = User::from_token(&req)?;
    user.alter_activation_status()
        .map(|usr| {
            let data = hashmap!["status" => "200", "message" => "User activation status changed"];
            let body = json!({
                "email": usr.email,
                "username": usr.username,
                "is_active": usr.is_active
            });

            respond(data, Some(body), None).unwrap()
        })
        .map_err(|e| e.into())
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

    match client.exchange_code(code) {
        Ok(token) => {
            let data = hashmap!["status" => "200",
            "message" => "sucess. Authorization token received"];

            respond(data, Some(token), None).unwrap()
        }
        Err(er) => err("403", er.to_string()),
    }
}

/// Registers users authenticated with google Oauth
/// This endpoint should be manually called with
/// the Oauth token received from the callback url (`/auth/callback`)
/// in the Authorization header
/// # url
/// `auth/register/social`
///
/// # method
/// get
///
/// # Arguments
/// `Authorization: Bearer`
pub async fn register_g_oauth(req: HttpRequest) -> HttpResponse {
    use serde_json::Value;

    let token_hdr = match Authorization::<Bearer>::parse(&req) {
        Ok(auth_header) => auth_header.into_scheme().to_string(),
        Err(e) => return err("400", e.to_string()),
    };

    let token = &token_hdr.split(' ').collect::<Vec<&str>>()[1];

    // Fetch user profile data
    let profile_url =
        env::var("GOOGLE_PROFILE_URL").expect("Missing the GOOGLE_PROFILE_URL env variable");

    let client = reqwest::Client::default();
    let mut headr = reqwest::header::HeaderMap::default();
    headr.append(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", token).parse().unwrap(),
    );
    let resp = client
        .get(&profile_url)
        .headers(headr)
        .send()
        .await
        .unwrap();
    if !resp.status().is_success() {
        return err(resp.status().as_str(), resp.json::<Value>().await.unwrap());
    }
    let res = &resp.json::<GoogleUser>().await.unwrap();
    // let j_res: Value = serde_json::from_str(&res).unwrap();

    match OauthGgUser::register_as_third_party(res) {
        Ok(data) => {
            let token = User::create_token(&res.email, None, "auth".into()).unwrap();

            if let Some(dt) = &data {
                // New oauth account

                respond(
                    hashmap!["status" => "201",
            "message" => "sucess. account created"],
                    Some(json!({
                        "email": & dt.0.email,
                        "token": &token,
                        "user": &data,
                    })),
                    None,
                )
                .unwrap()
            } else {
                // Existing
                respond(
                    hashmap!["status" => "200",
            "message" => "sucess. account updated"],
                    Some(json!({
                        "email": & res.email,
                        "token": &token,
                    })),
                    None,
                )
                .unwrap()
            }
        }

        // Registered regular account
        Err(e) => err("409", e.to_string()),
    }
}

/// Sends an account activation link to a user email
async fn send_activation_link(
    user_email: &str,
    user_name: Option<&str>,
    reset_link: &str,
    template: &str,
) -> Result<(), Error> {
    let context = get_context(user_name, reset_link);
    let mut username = "";

    let s = TEMPLATE
        .render(template, &context)
        .map_err(ErrorInternalServerError)?;
    if let Some(name) = user_name {
        username = name;
    }

    let mut mail = mail::Mail::new(user_email, username, "Email activation", &s)
        .await
        .map_err(ErrorInternalServerError)?;
    mail.send().await.map_err(ErrorInternalServerError)?;
    Ok(())
}

async fn generate_tokens(usr: &User) -> Result<(String, String), Error> {
    let auth_tk_duration = env::var("AUTH_TOKEN_DURATION")
        .unwrap_or_else(|e| {
            debug!("{}", e);
            "120".into()
        })
        .parse::<i64>()
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    let auth_token = User::create_token(&usr.email, Some(auth_tk_duration), "auth".into())
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    let rf_duration = env::var("REFRESH_TOKEN_DURATION")
        .unwrap_or_else(|e| {
            debug!("{}", e);
            "42600".into()
        })
        .parse::<i64>()
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;

    let refresh_tkn = User::create_token(&usr.email, Some(rf_duration), "refresh".into())
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok((auth_token, refresh_tkn))
}

/// Changes an account's user level
/// For a user to increase another's prilidge,
/// they must have a higher/equal priviledge than/equal to the one requested.
///
/// Ordinary users can't change another's access.
///
/// # url
/// `/auth/access`
///
/// ### Authentication Required
pub async fn change_user_access_level(
    req: HttpRequest,
    data: web::Json<NewUserLevel<'_>>,
) -> HttpResponse {
    if let Err(e) = data.validate() {
        return err("400", e.to_string());
    }
    let auth = match get_bearer(&req) {
        Ok(a) => a,
        Err(e) => return err("400", e),
    };
    let token = &auth.split(' ').collect::<Vec<&str>>()[1];

    let user = match User::alter_access_level(&data.into_inner(), token) {
        Ok(u) => u,
        Err(e) => {
            if e.eq("NotFound") {
                return err("404", e);
            }
            return err("401", e);
        }
    };
    let data = json!({
        "email": user.email,
        "access_level": user.access_level
    });
    let msg = hashmap!["status" => "200",
                       "message" => "sucess. user access level changed"];
    respond(msg, Some(data), None).unwrap().await.unwrap()
}

fn get_bearer(req: &HttpRequest) -> Result<String, String> {
    match Authorization::<Bearer>::parse(req) {
        Ok(auth_header) => Ok(auth_header.into_scheme().to_string()),
        Err(e) => Err(e.to_string()),
    }
}
