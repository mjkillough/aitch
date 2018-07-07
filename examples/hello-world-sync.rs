extern crate aitch;
extern crate http;

use aitch::{middlewares, Responder, ResponseBuilder};
use http::Request;

fn handler(_req: Request<Vec<u8>>, mut resp: ResponseBuilder) -> impl Responder {
    resp.body("Hello, world!".to_owned())
}

fn main() {
    let wrapped = middlewares::logging_handler(handler);

    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::Server::new(addr, wrapped).run();
}
