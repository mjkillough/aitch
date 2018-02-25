extern crate aitch;
extern crate futures;
extern crate http;

use aitch::ResponseBuilder;
use aitch::async::{AsyncBody, AsyncHandlerFunc, FutureResponse};
use http::Request;

fn handler(_req: &mut Request<AsyncBody>, mut resp: ResponseBuilder) -> FutureResponse<AsyncBody> {
    let v = "Hello from the future!".as_bytes().to_owned();
    let r = resp.body(v.into()).unwrap();
    Box::new(futures::future::ok(r))
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, AsyncHandlerFunc(handler))
        .run()
        .unwrap();
}
