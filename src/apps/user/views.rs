//! Handles views for User items
//!

use crate::apps::user::models::User;
use actix_web::{web, HttpResponse};

/// Registers a new user
///
/// # method
/// POST
///
/// # Returns
/// JSON of received User data
pub fn register_user(data: web::Json<User>) -> HttpResponse {
    println!("Data: {:#?}", data);
    HttpResponse::Ok().json(data.0)
}
