use super::{
    models::{NewJsonUser, SignInUser, User},
    views::{change_activation_status, get_user, login, register_user, verify},
};
use crate::apps::user::models::NewUser;

use actix_web::{http::StatusCode, test, web, App};
use std::{borrow::Cow, env};

const BASE: &str = "/api";
lazy_static! {
    #[derive(Debug)]
    pub static ref DB_URL: String = {
        let test_db = env::var("TEST_DATABASE_URL").expect("Missing env variable TEST_DATABASE_URL");
        env::set_var("DATABASE_URL", &test_db);
        assert_eq!(env::var("DATABASE_URL"), Ok(test_db));

        env::var("TEST_DATABASE_URL").expect("Missing database url")
    };
}

/// Creates new user
fn create_user(username: &str, password: &str, email: &str) {
    let username = Cow::Borrowed(username);
    let password = Cow::Borrowed(password);
    let email = Cow::Borrowed(email);

    let mut user = NewUser {
        username,
        password,
        access_level: Some(2),
    };

    if user.save(&email).is_ok() {}
}

/// Returns an auth for the encoded with the passed email
fn _auth_token(email: &str) -> String {
    let t = User::create_token(email, None, "auth".into()).unwrap();
    "Bearer ".to_owned() + &t
}

/// Returns a verification for the encoded with the passed email
fn _verif_token(email: &str) -> String {
    let t = User::create_token(email, None, "verification".into()).unwrap();
    "Bearer ".to_owned() + &t
}

/// Returns a verification Str for the encoded with the passed email
fn _verif_token_str(email: &str) -> String {
    User::create_token(email, None, "verification".into()).unwrap()
}
#[actix_rt::test]
async fn register_valid_user() {
    let _ = *DB_URL;
    // let url = "http://localhost:8888".to_owned() +
    let url = BASE.to_owned() + "/auth";

    let mut app = test::init_service(App::new().route(&url, web::post().to(register_user))).await;
    let body = NewJsonUser {
        email: Cow::Borrowed("userreg@f.co"),
        password: Cow::Borrowed("password"),
        username: Cow::Borrowed("userreg"),
        access_level: Some(2),
    };
    let req = test::TestRequest::post()
        .set_json(&body)
        .uri(&url)
        .to_request();
    let _resp = test::call_service(&mut app, req).await;

    // Email is sent on registration.
    // The req will err unless a mock of the email sending
    // is done.
    let user = User::find_by_email(&body.email).unwrap();
    assert_eq!(user[0].username, body.username.to_string());
}

#[actix_rt::test]
async fn register_invalid_user() {
    let _ = *DB_URL;
    let url = BASE.to_owned() + "/auth";

    let mut app = test::init_service(App::new().route(&url, web::post().to(register_user))).await;
    let invalid_named = NewJsonUser {
        email: Cow::Borrowed("user@f.co"),
        password: Cow::Borrowed("password"),
        username: Cow::Borrowed("sh"),
        access_level: Some(2),
    };
    let invalid_emailed = NewJsonUser {
        email: Cow::Borrowed("invalid_email"),
        password: Cow::Borrowed("password"),
        username: Cow::Borrowed("saddh"),
        access_level: Some(2),
    };

    let req = test::TestRequest::post()
        .set_json(&invalid_named)
        .uri(&url)
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    let resp2 = test::TestRequest::post()
        .set_json(&invalid_emailed)
        .uri(&url)
        .to_request();
    let resp2 = test::call_service(&mut app, resp2).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn register_user_twice() {
    let _ = *DB_URL;
    let url = BASE.to_owned() + "/auth";

    let mut app = test::init_service(App::new().route(&url, web::post().to(register_user))).await;
    let body = NewJsonUser {
        email: Cow::Borrowed("user@f.co"),
        password: Cow::Borrowed("password"),
        username: Cow::Borrowed("user1"),
        access_level: Some(2),
    };
    let req = test::TestRequest::post()
        .set_json(&body)
        .uri(&url)
        .to_request();
    let _resp = test::call_service(&mut app, req).await;
    let req = test::TestRequest::post()
        .set_json(&body)
        .uri(&url)
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[actix_rt::test]
async fn login_valid_user() {
    let url = BASE.to_owned() + "/auth/login";

    let mut app = test::init_service(App::new().route(&url, web::post().to(login))).await;

    let username = Cow::Borrowed("loggy");
    let password = Cow::Borrowed("password");
    let email = Cow::Borrowed("user@f.co");

    create_user(&username, &password, &email);
    let user = SignInUser {
        email: Some(email),
        username: Some(username),
        password,
    };
    let req = test::TestRequest::post()
        .set_json(&user)
        .uri(&url)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn login_nonexistent_username() {
    let url = BASE.to_owned() + "/auth/login";

    let mut app = test::init_service(App::new().route(&url, web::post().to(login))).await;

    let username = Cow::Borrowed("missing");
    let password = Cow::Borrowed("password");

    let user = SignInUser {
        email: None,
        username: Some(username),
        password,
    };
    let req = test::TestRequest::post()
        .set_json(&user)
        .uri(&url)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn login_nonexistent_email() {
    let url = BASE.to_owned() + "/auth/login";

    let mut app = test::init_service(App::new().route(&url, web::post().to(login))).await;

    let email = Cow::Borrowed("missing@co.cu");
    let password = Cow::Borrowed("password");

    let user = SignInUser {
        email: Some(email),
        username: None,
        password,
    };
    let req = test::TestRequest::post()
        .set_json(&user)
        .uri(&url)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn login_invalid_password() {
    let url = BASE.to_owned() + "/auth/login";

    let mut app = test::init_service(App::new().route(&url, web::post().to(login))).await;

    let username = Cow::Borrowed("loggy");
    let password = Cow::Borrowed("password");
    let email = Cow::Borrowed("user@f.co");

    let wrong_pass = "dookydooky!";

    create_user(&username, &password, &email);
    let user = SignInUser {
        email: Some(email),
        username: Some(username),
        password: Cow::Borrowed(wrong_pass),
    };
    let req = test::TestRequest::post()
        .set_json(&user)
        .uri(&url)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn fetch_registered_user() {
    let username = Cow::Borrowed("loaaiggy");
    let password = Cow::Borrowed("password");
    let email = Cow::Borrowed("useaaatr@f.co");

    create_user(&username, &password, &email);
    let user_id = User::find_by_email(&email).unwrap()[0].id;

    let token = _auth_token(&email);

    let url = BASE.to_owned() + &format!("/user/{}", user_id);

    let mut app =
        test::init_service(App::new().route("/api/user/{id}", web::get().to(get_user))).await;

    let req = test::TestRequest::get()
        .uri(&url)
        .header("Authorization", token)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    //    println!("res: {:?}", resp.response().body().as_ref().unwrap());

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn set_user_account_active() {
    let username = Cow::Borrowed("touso");
    let password = Cow::Borrowed("password");
    let email = Cow::Borrowed("userto@f.co");

    create_user(&username, &password, &email);
    let user = &User::find_by_email(&email).unwrap()[0];

    assert!(user.is_active);

    let token = _auth_token(&email);

    let url = BASE.to_owned() + "/auth/activate";

    let mut app =
        test::init_service(App::new().route(&url, web::get().to(change_activation_status))).await;

    let req = test::TestRequest::get()
        .uri(&url)
        .header("Authorization", token)
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    let deactivated_user = &User::find_by_email(&email).unwrap()[0];

    assert!(!deactivated_user.is_active);
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_rt::test]
async fn verify_user() {
    let username = Cow::Borrowed("toverif");
    let password = Cow::Borrowed("password");
    let email = Cow::Borrowed("toverif@f.co");

    create_user(&username, &password, &email);
    let user = &User::find_by_email(&email).unwrap()[0];

    assert!(!user.is_verified);

    let token = _verif_token_str(&email);

    let url = BASE.to_owned() + &format!("/auth/verify/{}", token);

    let mut app =
        test::init_service(App::new().route("api/auth/verify/{t}", web::get().to(verify))).await;

    let req = test::TestRequest::get().uri(&url).to_request();

    let resp = test::call_service(&mut app, req).await;
    let verified_user = &User::find_by_email(&email).unwrap()[0];

    assert!(verified_user.is_verified);
    assert_eq!(resp.status(), StatusCode::OK);
}
