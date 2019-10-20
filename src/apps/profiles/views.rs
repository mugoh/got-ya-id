//! Holds View-related functions for the Profile Module

use super::models::Profile;

use crate::core::response::{err, respond};
use crate::hashmap;

use actix_web::{web, HttpResponse};

/// Retrieves the profile matching the given user ID
///
/// # url
/// ## `/user/profile/{user_id}`
///
/// # method
///   GET
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

/// Retrieves all existing user profiles
///
/// # url
/// ## `/users/profiles`
///
/// # method
///     GET
pub fn get_all_profiles() -> HttpResponse {
    match Profile::retrieve_all() {
        Ok(vec) => {
            let data = hashmap!["status" => "200",
            "message" => "Success. Profiles retreived"];
            respond(data, Some(vec), None).unwrap()
        }
        Err(e) => err("500", e.to_string()),
    }
}
