use actix_web::error::ErrorConflict;
use actix_web::{web, Error, HttpResponse, Result};

use super::models::{Identification, NewIdentification};
use crate::{
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
    new_idt: web::Json<NewIdentification<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = new_idt.0.validate() {
        //return Ok(respond::<serde_json::Value>(hashmap!["status" => "400"], None, Some(&e.to_string())).unwrap());
        return Ok(err("400", e.to_string()));
    }
    new_idt.save().map_err(ErrorConflict).map(move |idt| {
        let res = hashmap!["status" => "201",
            "message" => "Success. Indentification created"];
        respond(res, Some(idt), None).unwrap()
    })
}

///Retrives a single Identification using its PK
pub async fn get_idt(pk: web::Path<i32>) -> Result<HttpResponse, Error> {
    let idt = Identification::find_by_id(*pk)?;

    let msg = hashmap!["status" => "201",
            "message" => "Success. Indentification created"];
    respond(msg, Some(idt), None).unwrap().await
}
