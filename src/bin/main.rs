use actix_web::{middleware, web, App, HttpServer};
//use env_logger;
use listenfd::ListenFd;
use std::{
    io,
    sync::{Arc, Mutex},
};

use got_ya_id::apps::api;
use got_ya_id::apps::user::{models::OClient, utils::create_oauth_client};

fn main() -> io::Result<()> {
    let mut listen_fd = ListenFd::from_env();

    // env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let data = OClient {
        client: create_oauth_client(),
    };
    let data = Arc::new(Mutex::new(data));

    let mut app = HttpServer::new(move || {
        App::new()
            .configure(api::api)
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(8192))
            .data(data.clone())
    });

    app = if let Some(listener) = listen_fd.take_tcp_listener(0).unwrap() {
        app.listen(listener)?
    } else {
        app.bind("127.0.0.1:8888")?
    };

    app.run()
}
