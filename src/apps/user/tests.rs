use super::{models::NewUser, views::*};
use actix_web::{http, test};

use std::borrow::Cow;

/// Should be able to register new user
const user: NewUser = NewUser {
    email: Cow::Borrowed("email"),
    password: Cow::Borrowed("pass"),
    username: Cow::Borrowed("username"),
};
#[test]
fn register_valid_user() {
    let req = test::TestRequest::default().set_json(&user);
    let res = test::block_on(register_user(req)).unwrap();

    assert_eq!(res.status(), http::StatusCode::OK);
}
