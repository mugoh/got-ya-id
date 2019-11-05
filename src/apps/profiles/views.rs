//! Holds View-related functions for the Profile Module

use super::models::{Profile, UpdtProfile};
use super::utils::extract_multipart_field;

use crate::apps::user::models::User;
use crate::core::response::{err, respond};
use crate::hashmap;

use actix_multipart::Multipart;
use actix_web::{error, web, Error, HttpResponse};

use futures::{Future, Stream};
use log::error as log_error;

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
            respond(data, Some(prof_vec.0.pop()), None).unwrap()
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

/// Updates the details of an existing User profile
///
/// # url
/// ## `/user/{id}/profile`
///
/// # Method
///     PUT
///
pub fn update_profile(data: web::Json<UpdtProfile>, id: web::Path<i32>) -> HttpResponse {
    match Profile::find_by_key(*id) {
        Ok(p_vec) => {
            let profile = &p_vec.0[0];
            let resp_data = hashmap!["status" => "200", "message" => "Success. Profile updated"];

            match profile.update(data.0) {
                Ok(p) => respond(resp_data, Some(p), None).unwrap(),
                Err(e) => err("500", e.to_string()),
            }
        }
        Err(e) => return err("404", e.to_string()),
    }
}

/// Uploads a file for use as the user's avatar
///
/// # url
/// ## `user/{id}/profile/avatar`
///
/// # Arguments
///
/// ## id
/// - ID of the user the avatar should belong it
///
/// ## multipart
/// - The mulitpart type of the request data containing the
///   upload file
///
/// # Method
///    PUT
pub fn upload_avatar(
    id: web::Path<i32>,
    multipart: Multipart,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let mut p: Option<User> = None;
    User::find_by_pk(*id)
        .map(|user_data| p = Some(user_data.0))
        .map_err(|e| HttpResponse::NotFound().json(e.to_string()));
    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| extract_multipart_field(field).into_stream())
        .flatten()
        .collect()
        .map(|upload_response| // [byte_size, url]
        {
            let _file_url = &upload_response[0].1;
            HttpResponse::Ok().json(p.unwrap().save_avatar(_file_url).unwrap())
        })
        .map_err(|e| {
            log_error!("File upload failed: {:?}", e);
            e
        })
}
