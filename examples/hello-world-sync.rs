extern crate aitch;
extern crate http;

use aitch::sync::{SyncBody, SyncHandlerFunc};
use aitch::ResponseBuilder;
use http::{Request, Response};

fn handler(
    _req: &mut Request<SyncBody>,
    mut resp: ResponseBuilder,
) -> http::Result<Response<SyncBody>> {
    resp.body("Hello, world!".as_bytes().to_owned())
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, SyncHandlerFunc(handler))
        .run()
        .unwrap();
}
