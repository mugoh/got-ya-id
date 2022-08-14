//! This module contains serialized JSON reponses
//! These are the responses given after a request
//! is processed
use serde::{Deserialize, Serialize};

use actix_web::{http::StatusCode, HttpResponse};
use serde_json::json;
use serde_json::Value;

use std::{collections::HashMap, hash::BuildHasher};
/// Response to User on Success
/// Deserialized to JSON
#[derive(Deserialize, Serialize)]
pub struct JsonResponse<T> {
    status: String,
    message: String,
    data: Option<T>,
}

impl<T> JsonResponse<T> {
    /// Returns a JSONResponse instance
    pub fn new(status: String, message: String, data: T) -> JsonResponse<T> {
        JsonResponse {
            status,
            message,
            data: Some(data),
        }
    }
}

/// Response to User on Failed request
/// Deserialized to JSON
#[derive(Deserialize, Serialize)]
pub struct JsonErrResponse<T> {
    status: String,
    errors: T,
}

impl<T> JsonErrResponse<T> {
    /// Returns a new JSON error instance
    pub fn new(status: String, errors: T) -> JsonErrResponse<T> {
        JsonErrResponse { status, errors }
    }
}

/// Response to User on Failed request
/// Deserialized to JSON
#[derive(Deserialize, Serialize)]
pub struct Response<'a, T> {
    status: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    message: Option<&'a str>,
}

impl<'a, T> Response<'a, T>
where
    T: serde::de::DeserializeOwned + Serialize,
{
    /// Returns a new JSON error instance
    pub fn err<'b>(status: &'b str, errors: &'b str) -> Response<'b, T> {
        Response {
            status,
            errors: Some(errors),
            data: None,
            message: None,
        }
    }

    pub fn success<'b>(status: &'b str, message: &'b str, data: Option<T>) -> Response<'b, T> {
        Response {
            status,
            errors: None,
            data,
            message: Some(message),
        }
    }
}

/// Constructs a HttpResponse
///
/// # Arguments
///
/// ## data: Hashmap<&'static str, &'static str>
/// message: Response message during 2** status (Success) response
///             Ignored for Error responses
/// status: Status code. e.g "200"
///
/// ## body: T
/// The data to be contained in the success reponse
/// It ought to be JSON Serializable
///
/// ## err: Option<&'a str>
/// The error to hold in the response for error type HttpResponses
///
/// # Returns
///  Result:
///
///  Ok: HttpResponse
///  Err: dyn std::error::Error
pub fn respond<'c, T, H: BuildHasher>(
    data: HashMap<&'c str, &'c str, H>,
    body: Option<T>,
    err: Option<&'c str>,
) -> Result<HttpResponse, ()>
//
where
    T: serde::de::DeserializeOwned,
    T: Serialize,
{
    let status = StatusCode::from_u16(data["status"].parse::<u16>().unwrap()).unwrap();
    if let Some(error_msg) = err {
        //   Ok(HttpResponse::build(status)
        //       .json(json!({"status": &status.to_string(), "error": err.unwrap()})))
        let res: Response<'c, Value> = Response::err(data["status"], error_msg);
        Ok(HttpResponse::build(status).json(res))
    } else {
        Ok(HttpResponse::build(status).json(Response::success(
            data["status"],
            &data.get("message").unwrap(),
            body,
        )))
    }
}

/// Constructs a HttpResponse
///
/// Like `respond` but not returning a Result
/// and has no `err` argument.
/// This is meant to be a replacement for `respond`
///
/// Calling `respond` with an error requires a type declaration,
/// but the aim of this was to provide SIMPLICITY in making
/// http responses.
///
///
/// # Arguments
///
/// ## data: Hashmap<&'static str, &'static str>
/// message: Response message during 2** status (Success) response
///             Ignored for Error responses
/// status: Status code. e.g "200"
///
/// ## body: T
/// The data to be contained in the success reponse
/// It ought to be JSON Serializable
///
/// # Returns
///  HttpResponse
pub fn respond2<'c, T, H: BuildHasher>(
    data: HashMap<&'c str, &'c str, H>,
    body: Option<T>,
) -> HttpResponse
//
where
    T: serde::de::DeserializeOwned,
    T: Serialize,
{
    let status = StatusCode::from_u16(data["status"].parse::<u16>().unwrap()).unwrap();

    HttpResponse::build(status).json(Response::success(
        data["status"],
        &data.get("message").unwrap(),
        body,
    ))
}
/// Gives a HttpResponse holding an error status
/// and the cause of request error
pub fn err<T: Serialize>(status: &'_ str, err: T) -> HttpResponse //serde_json::value::Value
{
    let status = StatusCode::from_u16(status.parse::<u16>().unwrap()).unwrap();

    let res = json!({"status": &status.to_string(), "errors": err});
    HttpResponse::build(status).json(res)
}
