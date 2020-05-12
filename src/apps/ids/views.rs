use actix_web::{web, Error, HttpRequest, HttpResponse, Result};

use super::models::{
    ClaimableIdentification, Identification, NewClaimableIdt, NewIdentification,
    UpdatableClaimableIdt, UpdatableIdentification,
};
use crate::{
    apps::user::models::User,
    core::response::{err, respond},
    hashmap,
};

use validator::Validate;

/// Receives a json NewIdentification data struct which is
/// used to POST a new Identification
///
/// # url
/// ``
/// # method
/// `POST`
pub async fn create_new_identification(
    mut new_idt: web::Json<NewIdentification<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_idt.0.validate() {
        //return Ok(respond::<serde_json::Value>(hashmap!["status" => "400"], None, Some(&e.to_string())).unwrap());
        return Ok(err("400", e.to_string()));
    }
    new_idt.save(&req).map_err(|e| e.into()).map(move |idt| {
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
/// its owner
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

/// Marks an Identification as `not found`
///
/// A found IDt is assumed to be marked as lost by
/// its owner
///
/// # Url
/// `/ids/lose/{key}`
///
/// # METHOD
/// `POST`
///
pub async fn lose_idt(pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    let idt = Identification::is_lost(pk.into_inner())?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification status marked NOT FOUND"];

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
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_data.validate() {
        return err("400", e.to_string()).await;
    };
    let idt = Identification::find_by_id(pk.into_inner())?;
    let saved = idt.update(&req, &new_data)?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identification updated"];

    respond(msg, Some(saved), None).unwrap().await
}

/// Retrieves Identifications belonging to the user
///
/// # Url
/// `/ids/mine`
///
/// # Method
/// GET
///
/// ## Authorization required
pub async fn get_user_idts(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = User::from_token(&req)?;
    let idts = Identification::show_mine(&user)?;

    let msg = hashmap!["status" => "200",
            "message" => "Success. Identifications retrieved"];

    respond(msg, Some(idts), None).unwrap().await
}

/// Allows a user to claim an Identification as belonging to them
///
/// # Url
/// `/ids/claim/mine`
///
/// # Method
/// `POST`
///
/// # Arguments
/// idt_data: The Identification information to be used in matching
/// the Identification of `idt_key` to the user sending the request
pub async fn claim_idt(idt_key: web::Path<&str>, req: HttpRequest) -> Result<HttpResponse, Error> {
    HttpResponse::build(actix_web::http::StatusCode::OK)
        .body("Hee")
        .await
}

/// Created a claim to an identification
/// The claim should have data similar-ish to the Identification
/// the owner of the claim is in search of.
///
/// The Identification the user wants <b>shouldn't neccesarily have been
/// found</b> at the time the claim is being created.
///
/// # Url
/// `/ids/claim/new`
///
/// # Method
/// `POST`
pub async fn create_idt_claim(
    req: HttpRequest,
    mut new_idt: web::Json<NewClaimableIdt<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_idt.validate() {
        return err("400", e.to_string()).await;
    }

    new_idt.save(&req).map_err(|e| e.into()).map(|res_data| {
        let msg = hashmap!["status" => "201",
            "message" => "Success. Claim saved"];
        respond(msg, Some(res_data), None).unwrap()
    })
}

/// Updates existing Claims
///
/// # Url
/// `idts/claim/{key}`
///
/// # Method
/// `PUT`
pub async fn update_idt_claim(
    pk: web::Path<i32>,
    req: HttpRequest,
    idt_data: web::Json<UpdatableClaimableIdt<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = idt_data.validate() {
        return err("400", e.to_string()).await;
    }
    ClaimableIdentification::find_by_id(*pk)
        .map_err(|e| e.into())
        .map(|claimed_idt| {
            claimed_idt
                .update(&req, idt_data.into_inner())
                .map(|updated| {
                    let msg = hashmap!["status" => "200",
            "message" => "Success. Claim updated"];

                    respond(msg, Some(updated), None).unwrap()
                })
                .map_err(|e| e.into())
        })
        .and_then(|res| res)
}

/// Retrieves Claimable Identifications by PK
///
/// # Url
/// `/ids/{pk}`
///
/// # Method
/// `GET`
pub async fn retrieve_claim(req: HttpRequest, pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    User::from_token(&req)?;

    let idt_claim = ClaimableIdentification::find_by_id(*pk)?;
    let msg = hashmap!["status" => "200",
            "message" => "Success. Claim  retrieved"];

    respond(msg, Some(idt_claim), None).unwrap().await
}
