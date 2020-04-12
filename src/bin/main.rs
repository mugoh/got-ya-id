use actix_web::{middleware, web, App, HttpServer};
use env_logger;
use listenfd::ListenFd;
use std::{env, io};

use got_ya_id::apps::api;

fn main() -> io::Result<()> {
    let mut listen_fd = ListenFd::from_env();

    env::set_var("RUST_LOG", "actix_todo=debug, actix-web=debug");
    env_logger::init();

    let mut app = HttpServer::new(|| {
        App::new()
            .configure(api::api)
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(8192))
    });

    app = if let Some(listener) = listen_fd.take_tcp_listener(0).unwrap() {
        app.listen(listener)?
    } else {
        app.bind("127.0.0.1:8888")?
    };

    app.run()
}
