//! Profile Views Module
use actix_web::{web, HttpResponse};

use crate::apps::profiles::models::Profile;
/// Retrieves the profile matching the given user ID
pub fn get_profile(id: web::Path<u32>) -> HttpResponse {
    let user_profile = Profile::find_by_key(*id);
}
