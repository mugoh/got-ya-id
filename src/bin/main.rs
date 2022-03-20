use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

//use env_logger;
use listenfd::ListenFd;
use std::{
    env, io,
    sync::{Arc, Mutex},
};

use tera::Tera;

use got_ya_id::{
    apps::{
        api,
        user::{models::OClient, utils::create_oauth_client},
    },
    diesel_cfg::config::seed_admin_user,
};

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let mut listen_fd = ListenFd::from_env();

    // env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let tera = Tera::new("src/templates/**/*").unwrap();
    seed_admin_user().await;

    let data = OClient {
        client: create_oauth_client(),
    };
    let data = Arc::new(Mutex::new(data));

    let mut app = HttpServer::new(move || {
        App::new()
            .configure(api::api)
            .wrap(Cors::new().supports_credentials().finish())
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(8192))
            .data(data.clone())
            .data(tera.clone())
    });

    app = if let Some(listener) = listen_fd.take_tcp_listener(0).unwrap() {
        app.listen(listener)?
    } else {
        let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let addr = format!("{}:{}", host, port);
        app.bind(&addr)?
    };

    app.run().await
}
