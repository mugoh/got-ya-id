use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use listenfd::ListenFd;
use std::io;

fn main() -> io::Result<()> {
    let mut listen_fd = ListenFd::from_env();

    let mut server = HttpServer::new(|| App::new().route("/", web::get().to(rooute)));

    server = if let Some(listener) = listen_fd.take_tcp_listener(0).unwrap() {
        server.listen(listener)?
    } else {
        server.bind("127.0.0.1:8888")?
    };

    server.run()
}

fn rooute(_req: HttpRequest) -> impl Responder {
    "Aha!"
}
