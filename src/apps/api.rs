//! This module holds the API routes configuration
//!
use actix_web::web;

use crate::apps::user;
/// Configures the app service
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/auth")
                    .route(web::post().to(user::views::register_user))
                    .route(web::get().to(|| "")),
            )
            .service(web::resource("/").route(web::get().to(|| "Aha"))),
    );
}
