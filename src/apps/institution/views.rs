use crate::{
    apps::user::models::User,
    core::response::{err, respond2 as respond},
    hashmap,
};

use super::models::ChangeableInst;

use actix_web::{web, Error, HttpRequest, HttpResponse, Result};

use validator::Validate;

/// Changes a User's institution to the name given.
///
/// # url:
/// `/institution/change`
///
/// # Method
/// `POST`
///
/// #### Authentication Required
///
/// ## Request Data Example
/// ```json
///     name: "name of new institution",
///     email: "email@thisinstituion.su.se"
/// ```
/// The email used should be an institutional email
/// for use in identification and verification of
/// user membership.
pub async fn change_institution(
    req: HttpRequest,
    data: web::Json<ChangeableInst<'_>>,
) -> Result<HttpResponse, Error> {
    if let Err(e) = data.validate() {
        return err("401", e.to_string()).await;
    }

    let user = User::from_token(&req)?;
    data.update(&user).await?;

    let res = hashmap!["status" => "200", "message" => "Success. Institution changed"];

    type H = std::collections::hash_map::RandomState;

    respond::<String, H>(res, None).await
}
