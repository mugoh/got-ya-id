use crate::{
    apps::user::models::User,
    core::response::{err, respond2 as respond},
    hashmap,
};

use super::models::{ChangeableInst, Institution, NewInstitution};

use actix_web::{web, Error, HttpRequest, HttpResponse, Result};

use validator::Validate;

/// Changes a User's institution to the name given.
///
/// # url:
/// `/institution/change`
///
/// # Method
/// `POST`
///
/// #### Authentication Required
///
/// ## Request Data Example
/// ```json
///     name: "name of new institution",
///     email: "email@thisinstituion.su.se"
/// ```
/// The email used should be an institutional email
/// for use in identification and verification of
/// user membership.
pub async fn change_institution(
    req: HttpRequest,
    data: web::Json<ChangeableInst<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = data.validate() {
        return err("400", e.to_string()).await;
    }

    let user = User::from_token(&req)?;
    data.update(&user).await?;

    let res = hashmap!["status" => "200", "message" => "Success. Institution changed"];

    type H = std::collections::hash_map::RandomState;

    respond::<String, H>(res, None).await
}

/// Creates new Insitution
///
/// # url:
/// `/institutions`
///
/// # Method
/// `POST`
///
/// #### Authorization Required
///
/// ## Request Data Example
/// ```json
/// {
///    name: "name of new institution",
///    town: "town of new institution",
///    country: "country of new institution",
///    description: "Some fancy stuff about the institution",
///    postal_address: "postal address of new institution"
///    }
/// ```
pub async fn create_institution(
    req: HttpRequest,
    new_insitution: web::Json<NewInstitution<'_>>,
) -> Result<HttpResponse, Error> {
    match new_insitution.validate() {
        Ok(_) => {}
        Err(e) => return err("400", e.to_string()).await,
    }
    User::from_token(&req)?;
    let insititution: Institution = new_insitution.save().await?;
    let msg = hashmap!["status" => "201", "message" => "Success. Institution created"];
    respond(msg, Some(insititution)).await
}

/// Gets all institutions
///
/// # url:
/// `/institutions`
///
/// # Method
/// `GET`
///
/// #### Authorization Required
pub async fn get_all_institutions(req: HttpRequest) -> Result<HttpResponse, Error> {
    User::from_token(&req)?;
    let institutions = Institution::get_all()?;
    let msg = hashmap!["status" => "200", "message" => "Success. Institutions retrieved"];
    respond(msg, Some(institutions)).await
}

/// Retrives an Institution's detail.
///
/// # url:
/// `/institutions/{id}`
///
/// # Method
/// `GET`
///
/// #### Authorization Required
pub async fn get_institution_detail(
    req: HttpRequest,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    User::from_token(&req)?;
    let institution = Institution::find_by_pk(id.into_inner())?;
    let msg = hashmap!["status" => "200", "message" => "Success. Institution retrieved"];
    respond(msg, Some(institution)).await
}
