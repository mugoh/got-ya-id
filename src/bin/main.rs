use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use listenfd::ListenFd;
use std::io;

use got_ya_id::apps::user;

fn main() -> io::Result<()> {
    let mut listen_fd = ListenFd::from_env();

    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath)
            .service(rooute)
            .service(web::resource("/auth").route(web::post().to(user::views::register_user)))
    });

    server = if let Some(listener) = listen_fd.take_tcp_listener(0).unwrap() {
        server.listen(listener)?
    } else {
        server.bind("127.0.0.1:8888")?
    };

    server.run()
}

#[get("/aha")]
fn rooute(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Aha!")
}
