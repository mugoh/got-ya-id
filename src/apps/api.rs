//! This module holds the API routes configuration
use actix_web::{guard, web, HttpResponse};

use crate::apps::{ids, profiles, user};

/// Configures the app service
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth")
                    .service(web::resource("").route(web::post().to(user::views::register_user)))
                    .service(web::resource("/login").route(web::post().to(user::views::login)))
                    .service(
                        web::resource("/google").route(web::get().to(user::views::google_auth)),
                    )
                    .service(
                        web::resource("/refresh/{refresh_token}")
                            .route(web::get().to(user::views::refresh_access_token)),
                    )
                    .service(
                        web::resource("/callback")
                            .route(web::get().to(user::views::google_auth_callback)),
                    )
                    .service(
                        web::resource("/access")
                            .route(web::post().to(user::views::change_user_access_level)),
                    )
                    .service(
                        web::resource("/logout/{token}").route(web::get().to(user::views::logout)),
                    )
                    .service(
                        web::resource("/verify/{token}").route(web::get().to(user::views::verify)),
                    )
                    .service(
                        web::resource("/password/reset/{token}")
                            .route(web::get().to(user::views::reset_password)),
                    )
                    .service(
                        web::resource("/password/request")
                            .route(web::post().to(user::views::send_reset_email)),
                    )
                    .service(
                        web::resource("/register/social")
                            .route(web::get().to(user::views::register_g_oauth)),
                    )
                    .service(
                        web::resource("/activation/send")
                            .route(web::post().to(user::views::send_account_activation_link)),
                    )
                    .service(
                        web::resource("/activate")
                            .route(web::patch().to(user::views::change_activation_status)),
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
                            .route(web::put().to(profiles::views::upload_avatar))
                            .route(web::get().to(profiles::views::retrieve_profile_avatar)),
                    ),
            )
            .service(web::scope("/users").service(
                web::resource("/profiles").route(web::get().to(profiles::views::get_all_profiles)),
            ))
            .service(web::scope("/ids").service(
                web::resource("/new").route(web::post().to(ids::views::create_new_identification)),
            ))
            .service(web::resource("/").route(web::get().to(|| HttpResponse::Ok().body("Aha"))))
            .default_service(
                // 404 GET
                web::resource("")
                    .route(web::get().to(|| {
                        HttpResponse::Ok().body("Oopsy! Coudn't find what you were looking for")
                    }))
                    // None GET
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            ),
    );
}
