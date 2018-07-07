extern crate aitch;
extern crate http;

use aitch::{logging_handler, Responder, ResponseBuilder, SimpleRouter};
use http::Request;

fn handler1(_req: Request<Vec<u8>>, mut resp: ResponseBuilder) -> impl Responder {
    resp.body("Handler 1!".as_bytes().to_owned())
}

fn handler2(_req: Request<Vec<u8>>, mut resp: ResponseBuilder) -> impl Responder {
    resp.body("Handler 2!".as_bytes().to_owned())
}

fn main() {
    let mut router = SimpleRouter::new();
    router.register_handler("/handler1", handler1);
    router.register_handler("/handler2", handler2);
    let handler = logging_handler(router);

    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::Server::new(addr, handler).run();
}
