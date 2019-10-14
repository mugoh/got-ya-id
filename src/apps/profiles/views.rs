//! Profile Views Module

use super::models::Profile;
use crate::core::response::{JsonErrResponse, JsonResponse};

use actix_web::{http::StatusCode, web, HttpResponse};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, error};

/// Retrieves the profile matching the given user ID
/// # method
///   GET
///
/// # url
/// : /user/profile/{user_id}
///
pub fn get_profile(id: web::Path<i32>) -> HttpResponse {
    match Profile::find_by_key(*id) {
        Ok(prof_vec) => HttpResponse::Found().json(&prof_vec[0]),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

/// Constructs a HttpResponse
///
/// # Arguments
/// data: Hashmap<String, String>
///     - message: Response message during 2** status (Success) response
///             Ignored for Error responses
///     - status:
///         Status code. e.g 200
/// body: T
///     The data to be contained in the success reponse
///     
///     - It ought to be translatable to a key-value pair in
///     the JSON reponse
///     - Valid types: json, Struct
/// err: U
///     The error to hold in the response for
///     `error` type HttpResponses
///
/// # Returns
///  Result
pub fn respond<T, U>(
    data: HashMap<&'static str, String>,
    body: Option<U>,
    err: Option<U>,
) -> Result<HttpResponse, Box<dyn error::Error>>
//
where
    for<'de> T: Deserialize<'de>,
    U: Serialize,
{
    //
    let status = StatusCode::from_u16(data["status"].parse::<u16>().unwrap()).unwrap();
    if err.is_some() {
        let res = JsonErrResponse::new(data.get("status").unwrap().into(), err.unwrap());
        Ok(HttpResponse::build(status).json(res))
    } else {
        let res = JsonResponse::new(
            data.get("status").unwrap().into(),
            data.get("message").unwrap().into(),
            body.unwrap(),
        );
        Ok(HttpResponse::build(status).json(res))
    }
}
