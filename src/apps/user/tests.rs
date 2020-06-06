use super::{
    models::{NewJsonUser, User},
    views::register_user,
};

use actix_web::{http, test, web, App};
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

async fn _register_valid_user() {
    let _ = *DB_URL;
    // let url = "http://localhost:8888".to_owned() +
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

    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    assert_eq!(resp2.status(), http::StatusCode::BAD_REQUEST);
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

    assert_eq!(resp.status(), http::StatusCode::CONFLICT);
}
