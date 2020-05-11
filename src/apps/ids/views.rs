use actix_web::error::ErrorConflict;
use actix_web::{web, Error, HttpRequest, HttpResponse, Result};

use super::models::{Identification, NewIdentification, UpdatableIdentification};
use crate::{
    apps::user::models::User,
    core::response::{err, respond},
    hashmap,
};

use validator::Validate;

use actix_web::http::header::Header;
use actix_web_httpauth::headers::authorization::Authorization;
use actix_web_httpauth::headers::authorization::Bearer;

/// Receives a json NewIdentification data struct which is
/// used to POST a new Identification
///
/// # url
/// ``
/// # method
/// `POST`
pub async fn create_new_identification(
    new_idt: web::Json<NewIdentification<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_idt.0.validate() {
        //return Ok(respond::<serde_json::Value>(hashmap!["status" => "400"], None, Some(&e.to_string())).unwrap());
        return Ok(err("400", e.to_string()));
    }
    new_idt.save().map_err(ErrorConflict).map(move |idt| {
        let res = hashmap!["status" => "201",
            "message" => "Success. Identification created"];
        respond(res, Some(idt), None).unwrap()
    })
}

///Retrives a single Identification using its PK
///
/// # url
/// `/ids/{id_key}`
///
/// # Method
///  `GET`
pub async fn get_idt(pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    let idt = Identification::find_by_id(*pk)?;

    let msg = hashmap!["status" => "201",
            "message" => "Success. Identification retrived"];
    respond(msg, Some(idt), None).unwrap().await
}

/// Retrieves all existing Identifications
/// # Url
/// `/ids`
///
/// # Method
/// `GET`
pub async fn get_all_idts() -> Result<HttpResponse, Error> {
    let data = Identification::retrieve_all()?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Identifications retrieved"];

    respond(msg, Some(data), None).unwrap().await
}

/// Marks an Identification as `found`
///
/// A found IDt is assumed to have been acquired by
/// the its owner
///
/// # Url
/// `/ids/found/{key}`
///
/// # METHOD
/// `POST`
///
pub async fn is_now_found(pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    let idt = Identification::mark_found(pk.into_inner())?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification status marked FOUND"];

    respond(msg, Some(idt), None).unwrap().await
}

/// Updates data in a given Identification
///
/// # Url
/// `/ids/{key}`
///
/// # Method
/// `PUT`
pub async fn update_idt(
    pk: web::Path<i32>,
    new_data: web::Json<UpdatableIdentification<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_data.validate() {
        return err("400", e.to_string()).await;
    };
    let idt = Identification::find_by_id(pk.into_inner())?;
    let saved = idt.update(&new_data)?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification updated"];

    respond(msg, Some(saved), None).unwrap().await
}

/// Retrieves Identifications belonging to the user
///
/// # Url
/// `/idts/mine`
///
/// # Method
/// GET
///
/// ## Authorization required
pub async fn get_user_idts(req: HttpRequest) -> Result<HttpResponse, Error> {
    let auth = extract_auth_header(&req)?;

    let token = &auth.split(' ').collect::<Vec<&str>>()[1];

    let user = User::from_token(token);

    HttpResponse::build(actix_web::http::StatusCode::OK)
        .body("ok")
        .await
}

/// Extracts the bearer authorization header
fn extract_auth_header(req: &HttpRequest) -> Result<String, actix_web::error::ParseError> {
    let auth_header = Authorization::<Bearer>::parse(req)?;
    Ok(auth_header.into_scheme().to_string())
}
