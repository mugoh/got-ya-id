use actix_web::{
    get, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use std::io;

mod apps;
/// User Object
/// Holds user data
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: u32,
    name: String,
}

fn main() -> io::Result<()> {
    let mut listen_fd = ListenFd::from_env();

    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath)
            .service(rooute)
            .service(web::resource("/auth").route(web::post().to(register_user)))
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

/// Registers a new user
///
/// # method
/// POST
///
/// # Returns
/// JSON of received User data
pub fn register_user(data: web::Json<User>) -> HttpResponse {
    println!("Data: {:#?}", data);
    HttpResponse::Ok().json(data.0)
}
