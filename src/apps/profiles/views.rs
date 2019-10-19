//! Profile Views Module

use super::models::Profile;

use crate::core::response::{err, respond};
use crate::hashmap;

use actix_web::{web, HttpResponse};

/// Retrieves the profile matching the given user ID
/// # method
///   GET
///
/// # url
/// : /user/profile/{user_id}
///
pub fn get_profile(id: web::Path<i32>) -> HttpResponse {
    let res = match Profile::find_by_key(*id) {
        Ok(mut prof_vec) => {
            let data = hashmap!["status" => "200",
            "message" => "Success. Profile retreived"];
            respond(data, Some(prof_vec.pop()), None).unwrap()
        }
        Err(e) => err("404", e.to_string()),
    };
    res
}
