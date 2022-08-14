use crate::{
    apps::user::models::User,
    core::response::{err, respond2 as respond},
    hashmap,
};

use super::models::{
    Institution, NewInstitution, UpdatableInstitution, UpdatableJsonUserInsitution,
};

use actix_web::{web, Error, HttpRequest, HttpResponse, Result};

use validator::Validate;

/// Changes a User's institution to the name given.
///
/// # url:
/// `/institutions/user/change`
///
/// # Method
/// `POST`
///
/// #### Authentication Required
///
/// ## Request Data Example
/// ```json
///     user_id: "Id of the user",
///     institution_id: "Id of the new institution"
/// ```
///
/// To verify if a user belongs to the institution,
/// we verify if any of the user's emails belong to the institution.
///
/// So a user making this request should have a verirified
/// institutional email for use in identification and
/// verification of institution membership.
pub async fn change_institution(
    req: HttpRequest,
    data: web::Json<UpdatableJsonUserInsitution>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = data.validate() {
        return err("400", e).await;
    }

    let user = User::from_token(&req)?;
    Institution::change_user_institution(&user, &data).await?;

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
    if let Err(e) = new_insitution.validate() {
        return err("400", e).await;
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
    let institution = Institution::find_by_pk(id.into_inner()).await?;
    let msg = hashmap!["status" => "200", "message" => "Success. Institution retrieved"];
    respond(msg, Some(institution)).await
}

/// Update Insitution detail.
///
/// # url:
/// `/institutions/{id}`
///
/// # Method
/// `PUT`
///
/// #### Authorization Required
/// ## Request Data Example
/// ```json
/// {
///   name: "name of new institution",
///   town: "town of new institution",
///   country: "country of new institution",
///   description: "Some fancy stuff about the institution",
///   postal_address: "postal address of new institution"
///   }
/// ```
pub async fn update_institution(
    pk: web::Path<i32>,
    new_data: web::Json<UpdatableInstitution<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_data.validate() {
        return err("400", e).await;
    }
    User::from_token(&req)?;

    let mut insitution = Institution::find_by_pk(pk.into_inner()).await?;
    insitution = insitution.update(&new_data).await?;
    let msg = hashmap!["status" => "200", "message" => "Success. Institution updated"];
    respond(msg, Some(insitution)).await
}
