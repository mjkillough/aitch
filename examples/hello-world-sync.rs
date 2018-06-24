extern crate aitch;
extern crate http;

use aitch::ResponseBuilder;
use http::{Request, Response};

fn handler(
    _req: &mut Request<Vec<u8>>,
    mut resp: ResponseBuilder,
) -> http::Result<Response<Vec<u8>>> {
    resp.body("Hello, world!".as_bytes().to_owned())
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::Server::new(addr, handler).run().unwrap();
}
