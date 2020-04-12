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
                        web::resource("/google").route(web::get().to(user::views::google_auth)),
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
                    )
                    .service(
                        web::resource("/activate")
                            .route(web::patch().to_async(user::views::change_activation_status)),
                    ),
            )
            .service(
                web::scope("/user")
                    .service(
                        web::resource("/{id}/profile")
                            .route(web::get().to(profiles::views::get_profile))
                            .route(web::put().to(profiles::views::update_profile)),
                    )
                    .service(web::resource("/{id}").route(web::get().to(user::views::get_user)))
                    .service(
                        web::resource("/{id}/profile/avatar")
                            .route(web::put().to_async(profiles::views::upload_avatar)),
                    ),
            )
            .service(web::scope("/users").service(
                web::resource("/profiles").route(web::get().to(profiles::views::get_all_profiles)),
            ))
            .service(web::resource("/").route(web::get().to(|| "Aha")))
            .default_service(
                // 404 GET
                web::resource("")
                    .route(web::get().to(|| "Oopsy! Coudn't find what you were looking for"))
                    // None GET
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            ),
    );
}
