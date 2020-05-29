//! This module holds the API routes configuration
use actix_web::{guard, web, HttpResponse};

use crate::apps::{
    email::views as email, ids::views as ids, profiles::views as profiles, user::views as user,
};

/// Configures the app service
pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth")
                    .service(web::resource("").route(web::post().to(user::register_user)))
                    .service(web::resource("/login").route(web::post().to(user::login)))
                    .service(web::resource("/google").route(web::get().to(user::google_auth)))
                    .service(
                        web::resource("/refresh/{refresh_token}")
                            .route(web::get().to(user::refresh_access_token)),
                    )
                    .service(
                        web::resource("/callback").route(web::get().to(user::google_auth_callback)),
                    )
                    .service(
                        web::resource("/access")
                            .route(web::post().to(user::change_user_access_level)),
                    )
                    .service(web::resource("/logout/{token}").route(web::get().to(user::logout)))
                    .service(web::resource("/verify/{token}").route(web::get().to(user::verify)))
                    .service(
                        web::resource("/password/reset/{token}")
                            .route(web::get().to(user::reset_password)),
                    )
                    .service(
                        web::resource("/password/request")
                            .route(web::post().to(user::send_reset_email)),
                    )
                    .service(
                        web::resource("/register/social")
                            .route(web::get().to(user::register_g_oauth)),
                    )
                    .service(
                        web::resource("/activation/send")
                            .route(web::post().to(user::send_account_activation_link)),
                    )
                    .service(
                        web::resource("/activate")
                            .route(web::patch().to(user::change_activation_status)),
                    ),
            )
            .service(
                web::scope("/user")
                    .service(
                        web::resource("/profile/{id}")
                            .route(web::get().to(profiles::get_profile))
                            .route(web::put().to(profiles::update_profile)),
                    )
                    .service(web::resource("/{id}").route(web::get().to(user::get_user)))
                    .service(
                        web::resource("/profile/avatar/{id}")
                            .route(web::put().to(profiles::upload_avatar))
                            .route(web::get().to(profiles::retrieve_profile_avatar)),
                    ),
            )
            .service(web::scope("/users").service(
                web::resource("/profiles").route(web::get().to(profiles::get_all_profiles)),
            ))
            .service(
                web::scope("/email")
                    .service(web::resource("/new").route(web::post().to(email::add_email))),
            )
            .service(
                web::scope("/ids")
                    .service(
                        web::resource("/new").route(web::post().to(ids::create_new_identification)),
                    )
                    .service(web::resource("/mine").route(web::get().to(ids::get_user_idts)))
                    .service(web::resource("claim/mine").route(web::post().to(ids::claim_idt)))
                    .service(
                        web::resource("/posted/me").route(web::get().to(ids::get_user_posted_idts)),
                    )
                    .service(
                        web::resource("/claim/user").route(web::get().to(ids::retrieve_user_claim)),
                    )
                    .service(
                        web::resource("/claim/{pk}")
                            .route(web::put().to(ids::update_idt_claim))
                            .route(web::get().to(ids::retrieve_claim)),
                    )
                    .service(web::resource("/claim").route(web::post().to(ids::create_idt_claim)))
                    .service(
                        web::resource("/{pk}")
                            .route(web::get().to(ids::get_idt))
                            .route(web::put().to(ids::update_idt)),
                    )
                    .service(web::resource("/all").route(web::get().to(ids::get_all_idts)))
                    .service(web::resource("/missing").route(web::get().to(ids::get_missing_idts)))
                    .service(web::resource("/found").route(web::get().to(ids::get_found_idts)))
                    .service(web::resource("/lose/{pk}").route(web::post().to(ids::lose_idt)))
                    .service(web::resource("/found/{pk}").route(web::post().to(ids::is_now_found))),
            )
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
