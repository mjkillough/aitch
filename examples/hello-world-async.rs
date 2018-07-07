extern crate aitch;
extern crate futures;
extern crate http;

use aitch::{middlewares, Responder, ResponseBuilder};
use futures::IntoFuture;
use http::Request;

fn handler(_req: Request<Vec<u8>>, mut resp: ResponseBuilder) -> impl Responder {
    let vec = "Hello from the future!".as_bytes().to_owned();
    resp.body(vec).into_future()
}

fn main() {
    let wrapped = middlewares::logging_handler(handler);

    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::Server::new(addr, wrapped).run();
}
