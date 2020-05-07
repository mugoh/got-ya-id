use super::models::NewUser;
use super::views::*;

use actix_web::{http, test, web, App};
use std::{borrow::Cow, env};

const base: &str = "/api";
lazy_static! {
    static ref DB_URL: String = {
        let test_db = env::var("TESTi_DATABASE_URL").expect("Missing database url");
        env::set_var("DATABASE_URL", test_db);
        env::var("TEST_DATABASE_URL").expect("Missing database url")
    };
}

#[actix_rt::test]
async fn register_valid_user() {
    &DB_URL;
    // let url = "http://localhost:8888".to_owned() +
    let url = base.to_owned() + "/auth";
    println!("URL: {}", url);

    let mut app = test::init_service(App::new().route(&url, web::post().to(register_user))).await;
    let body = NewUser {
        email: Cow::Borrowed("user@f.co"),
        password: Cow::Borrowed("password"),
        username: Cow::Borrowed("user"),
        access_level: None,
    };
    let req = test::TestRequest::post()
        .set_json(&body)
        .uri(&url)
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    println!("RES: {:?}", resp);
    assert!(resp.status().is_success());
}
//test::TestRequest::post().set_json().to_request
