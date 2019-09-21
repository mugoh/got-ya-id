//! This module holds the API routes configuration
//!
use actix_web::{guard, web, HttpResponse};

use crate::apps::user;
/// Configures the app service
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/auth")
                    .route(web::post().to(user::views::register_user))
                   // .route(web::get().to(|| "")),
            )
            .service(web::resource("/").route(web::get().to(|| "Aha")))
            .default_service(
                // 404 GET
                web::resource("")
                    .route(web::get().to(|| " Not Found"))
                    // None GET
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            ),
    );
}
