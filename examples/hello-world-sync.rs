extern crate aitch;
extern crate http;

use aitch::{Responder, ResponseBuilder};
use http::Request;

fn handler(_req: &mut Request<Vec<u8>>, mut resp: ResponseBuilder) -> impl Responder<Vec<u8>> {
    resp.body("Hello, world!".as_bytes().to_owned())
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::Server::new(addr, handler).run();
}
