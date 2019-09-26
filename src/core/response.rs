//! This module contains serialized JSON reponses
//! These are the responses given after a request
//! is processed
use serde::{Deserialize, Serialize};

/// Response to User on Success
/// Deserialized to JSON
#[derive(Deserialize, Serialize, Debug)]
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
#[derive(Deserialize, Serialize, Debug)]
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
