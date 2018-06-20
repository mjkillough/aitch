extern crate aitch;
extern crate http;

use aitch::sync::{SyncBody, SyncHandlerFunc};
use aitch::{ResponseBuilder, SimpleRouter};
use http::{Request, Response};

fn handler1(
    _req: &mut Request<SyncBody>,
    mut resp: ResponseBuilder,
) -> http::Result<Response<SyncBody>> {
    resp.body("Handler 1!".as_bytes().to_owned())
}

fn handler2(
    _req: &mut Request<SyncBody>,
    mut resp: ResponseBuilder,
) -> http::Result<Response<SyncBody>> {
    resp.body("Handler 2!".as_bytes().to_owned())
}

fn main() {
    let mut router = SimpleRouter::new();
    router.register_handler("/handler1", SyncHandlerFunc(handler1));
    router.register_handler("/handler2", SyncHandlerFunc(handler2));

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, router).run().unwrap();
}
