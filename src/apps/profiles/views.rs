//! Holds View-related functions for the Profile Module

use super::models::{Profile, UpdtProfile};
use super::utils::make_temp_file;

use crate::apps::user::models::User;
use crate::core::{response::{err, respond}, py_interface::create_py_mod};
use crate::hashmap;

use actix_multipart::Multipart;
use actix_web::{ web, Error, HttpResponse};

use futures::{StreamExt, TryStreamExt};
use std::io::Write;


/// Retrieves the profile matching the given user ID
///
/// # url
/// ## `/user/profile/{user_id}`
///
/// # method
///   GET
///
pub fn get_profile(id: web::Path<i32>) -> HttpResponse {
    match Profile::find_by_key(*id) {
        Ok(mut prof_vec) => {
            let data = hashmap!["status" => "200",
            "message" => "Success. Profile retreived"];
            respond(data, Some(prof_vec.0.pop()), None).unwrap()
        }
        Err(e) => err("404", e.to_string()),
    }
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
        Err(e) => err("404", e.to_string()),
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
/// - ID of the user the avatar should belong to
///
/// ## multipart
/// - The mulitpart type of the request data containing the
///   upload file
///
/// # Method
///    PUT
pub async fn upload_avatar(
    id: web::Path<i32>,
    mut multipart: Multipart,
) -> Result<HttpResponse, Error> {
    let mut path = "empty".into();

    let user = match User::find_by_pk(*id, None){
        Ok(usr) => usr.0,
    Err(e) =>return  Ok(err("400", e.to_string()))
    };

    // iterate over multipart stream
    while let Ok(Some(mut field)) = multipart.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap().to_string();

        // let filepath = format!("./tmp/");
        // File::create is blocking operation, use threadpool
        
        // let  f = web::block(|| std::fs::File::create(filepath))
        //    .await
        //    .unwrap();
        let  ( mut f, filepath) = web::block(|| make_temp_file(Some(filename))).await?;
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }

        path = create_py_mod(filepath).expect("Initiating file send failed");
    }
   Ok(HttpResponse::Ok().body(path))

}

/// Retrieves an avatar url of a user profile
///
/// # url
/// ## `/user/{user_id}/profile/avatar`
///
/// # method
/// GET
pub fn retrieve_profile_avatar(id: web::Path<i32>) -> HttpResponse {
    let user = match User::find_by_pk(*id, None) {
        Ok(usr) => usr.0,
        Err(e) => return err("404", e.to_string()),
    };

    match user.get_avatar() {
        Ok(avatar) => respond(
            hashmap!["status" => "200", "message" => "Success. Avatar retrieved"],
            avatar,
            None,
        )
        .unwrap(),
        Err(e) => err("500", e.to_string()),
    }
}
