use crate::{
    apps::user::models::{User, UserEmail},
    core::response::{err, respond},
    hashmap,
};

use super::models::{Email, NewEmail};

use actix_web::{web, Error, HttpRequest, HttpResponse, Result};
use validator::Validate;

/// Adds a new email for a user account.
///
/// # url `/emails/new`
///
/// # Method: `Post`
///
/// #### Authentication required
///
/// ## Request data format
/// ```none
/// let new_email = NewEmail {email: "donuty@email.nuts"}
/// ```
pub async fn add_email(
    req: HttpRequest,
    mut new_email: web::Json<NewEmail<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_email.validate() {
        return err("400", e.to_string()).await;
    }

    let user = User::from_token(&req)?;
    new_email.user_id = user.id;
    let saved_email = new_email.save()?;

    let data = hashmap!["status"=> "201", "message"=> "Success. Email added"];
    respond(data, Some(saved_email), None).unwrap().await
}

/// Dissacciates an email with a user account.
///
/// # url `/emails/remove`
///
/// # Method: `PUT`
///
/// #### Authentication required
///
/// ## Request data format
/// ```none
/// let email = UserEmail {email: "donuty@email.nuts"}
/// ```
pub async fn remove_email(
    req: HttpRequest,
    email: web::Json<UserEmail<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = email.validate() {
        return err("400", e.to_string()).await;
    }

    let user = User::from_token(&req)?;

    let removed_e = Email::remove(&email.into_inner().email, user.id)?;

    let data = hashmap!["status"=> "200", "message"=> "Success. Email removed"];
    respond(data, Some(removed_e), None).unwrap().await
}
