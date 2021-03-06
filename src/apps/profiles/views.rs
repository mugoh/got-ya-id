//! Holds View-related functions for the Profile Module

use super::models::{Profile, UpdtProfile};
use super::utils::make_temp_file;

use crate::apps::user::models::User;
use crate::core::{
    py_interface::create_py_mod,
    response::{err, respond},
};
use crate::hashmap;

use actix_multipart::Multipart;
use actix_web::{http::StatusCode, web, Error, HttpRequest, HttpResponse, Result};

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
/// #### Authentication Required
pub async fn get_profile(id: web::Path<i32>, req: HttpRequest) -> Result<HttpResponse, Error> {
    User::decode_auth_header(&req)?;

    match Profile::find_by_key(*id) {
        Ok(mut prof_vec) => {
            let data = hashmap!["status" => "200",
            "message" => "Success. Profile retreived"];
            respond(data, Some(prof_vec.0.pop()), None).unwrap().await
        }
        Err(e) => err("404", e.to_string()).await,
    }
}

/// Retrieves all existing user profiles
///
/// # url
/// ## `/users/profiles`
///
/// # method
/// GET
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
/// ## `/user/profile/{id}`
///
/// # Method
/// PUT
///
/// #### Authentication Required
pub async fn update_profile(
    data: web::Json<UpdtProfile<'_>>,
    id: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let this_user = User::from_token(&req)?;

    match Profile::find_by_key(*id) {
        Ok(p_vec) => {
            let profile = &p_vec.0[0];

            if profile.user_id != this_user.id {
                return err("401", "You are not allowed to do that".to_string()).await;
            }

            match profile.update(data.0) {
                Ok(p) => {
                    let resp_data = hashmap!["status" => "200",
                    "message" => "Success. Profile updated"];
                    respond(resp_data, Some(p), None).unwrap().await
                }
                Err(e) => err("500", e.to_string()).await,
            }
        }
        Err(e) => err("404", e.to_string()).await,
    }
}

/// Uploads a file for use as the user's avatar
///
/// # url
/// ## `user/profile/avatar/{user_id}`
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
///
/// #### Authentication Required
pub async fn upload_avatar(
    pk: web::Path<i32>,
    mut multipart: Multipart,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut path = "".into();
    let user = User::from_token(&req)?;

    if user.id != pk.into_inner() {
        return err("401", "Oopsy. You are not allowed to do that".to_string()).await;
    }

    // iterate over multipart stream
    while let Ok(Some(mut field)) = multipart.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap().to_string();

        // File::create is blocking operation, use threadpool

        let (mut f, filepath) = web::block(|| make_temp_file(Some(filename))).await?;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }

        path = create_py_mod(filepath, "got_ya_id/avatars/")?;
    }

    if !path.is_empty() {
        match user.save_avatar(&path) {
            Ok(_) => Ok(HttpResponse::Ok().json(path)),
            Err(e) => {
                Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string()))
            }
        }
    } else {
        Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
            .json("File upload failed, big man. File upload failed"))
    }
}

/// Retrieves an avatar url of a user profile
///
/// # url
/// ## `/user/profile/avatar/{user_id}`
///
/// # method
/// GET
///
/// #### Authentication Required
pub async fn retrieve_profile_avatar(
    id: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user = User::from_token(&req)?;

    match User::find_by_pk(*id, None) {
        Ok(usr) => {
            if usr.0.id != user.id {
                return err("401", "Permission denied".to_string()).await;
            }
        }
        Err(e) => return err("404", e.to_string()).await,
    };

    match user.get_avatar() {
        Ok(avatar) => {
            respond(
                hashmap!["status" => "200", "message" => "Success. Avatar retrieved"],
                avatar,
                None,
            )
            .unwrap()
            .await
        }
        Err(e) => err("500", e.to_string()).await,
    }
}
