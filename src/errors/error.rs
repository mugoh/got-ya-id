//! Custom Errors

use actix_web::{http::StatusCode, HttpResponse, ResponseError};

use serde::Serialize;
use serde_json::{json, to_string_pretty};

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Custom Error to send in http responses
#[derive(Serialize, Debug)]
pub struct ResError {
    /// Error message
    msg: String,
    /// Status code
    status: u16,
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

impl std::convert::From<diesel::result::Error> for ResError {
    fn from(er: diesel::result::Error) -> Self {
        let msg = er.to_string();
        let status = if msg.contains("NotFound") { 404 } else { 500 };
        Self { msg, status }
    }
}
