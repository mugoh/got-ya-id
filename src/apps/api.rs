//! This module holds the API routes configuration
//!
use actix_web::{guard, web, HttpResponse};

use crate::apps::{profiles, user};
/// Configures the app service
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth")
                    .service(
                        web::resource("").route(web::post().to(user::views::register_user)), // .route(web::get().to(|| "")),
                    )
                    .service(
                        web::resource("/login").route(web::post().to_async(user::views::login)),
                    )
                    .service(
                        web::resource("/verify/{token}")
                            .route(web::get().to_async(user::views::verify)),
                    )
                    .service(
                        web::resource("/password/reset/{token}")
                            .route(web::patch().to_async(user::views::reset_password)),
                    )
                    .service(
                        web::resource("/password/request")
                            .route(web::post().to_async(user::views::send_reset_email)),
                    ),
            )
            .service(web::scope("/user").service(
                //
                web::resource("/profile").route(web::post().to(profiles::views::get_profile)),
            ))
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
