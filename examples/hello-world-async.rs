extern crate aitch;
extern crate futures;
extern crate http;

use aitch::{Responder, ResponseBuilder};
use futures::IntoFuture;
use http::Request;

fn handler(_req: &mut Request<Vec<u8>>, mut resp: ResponseBuilder) -> impl Responder<Vec<u8>> {
    let vec = "Hello from the future!".as_bytes().to_owned();
    resp.body(vec).into_future()
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::Server::new(addr, handler).run();
}
