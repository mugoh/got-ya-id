//! Custom Errors

use actix_web::{http::StatusCode, HttpResponse, ResponseError};

use serde::Serialize;
use serde_json::{json, to_string_pretty};

use std::{
    convert::From,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// Custom Error to send in http responses
#[derive(Serialize, Debug)]
pub struct ResError {
    /// Error message
    pub msg: String,
    /// Status code
    pub status: u16,
}

impl ResError {
    /// Create a new instance of the ResError
    pub fn new(msg: String, status: u16) -> Self {
        Self { msg, status }
    }

    /// Returns an Unauthorized error response
    /// Status: 401
    pub fn unauthorized() -> Self {
        Self {
            msg: "Oopsy! It seems you are not allowed to do that".into(),
            status: 401,
        }
    }

    /// Returns a NOT FOUND error response
    /// Status: 404
    pub fn not_found() -> Self {
        Self {
            msg: "Resource not found".into(),
            status: 404,
        }
    }
}

impl ResponseError for ResError {
    /// Builds the sendable response
    fn error_response(&self) -> HttpResponse {
        let er = json!({"errors": self.msg,"status": self.status});
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json2(&er)
    }
}

impl Display for ResError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl From<diesel::result::Error> for ResError {
    fn from(er: diesel::result::Error) -> Self {
        let msg = er.to_string();
        let status = if msg.contains("NotFound") { 404 } else { 500 };
        Self { msg, status }
    }
}

impl From<jsonwebtoken::errors::Error> for ResError {
    fn from(er: jsonwebtoken::errors::Error) -> Self {
        let msg = er.to_string();
        let status = 401;

        Self { msg, status }
    }
}

impl From<actix_web::error::ParseError> for ResError {
    fn from(er: actix_web::error::ParseError) -> Self {
        let msg = er.to_string();
        let status = 400;

        Self { msg, status }
    }
}

impl From<lettre::smtp::error::Error> for ResError {
    fn from(er: lettre::smtp::error::Error) -> Self {
        let msg = er.to_string();
        let status = 500;

        Self { msg, status }
    }
}

impl From<tera::Error> for ResError {
    fn from(er: tera::Error) -> Self {
        let msg = er.to_string();
        let status = 500;

        Self { msg, status }
    }
}
