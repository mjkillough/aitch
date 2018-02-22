extern crate aitch;
extern crate http;

use aitch::{Handler, HandlerFunc, Request, Response, ResponseBuilder};

fn handler1(_req: &mut Request, mut resp: ResponseBuilder) -> http::Result<Response> {
    resp.body("Hello world 1!".as_bytes().to_owned())
}

fn handler2(_req: &mut Request, mut resp: ResponseBuilder) -> http::Result<Response> {
    resp.body("Hello world 2!".as_bytes().to_owned())
}

fn main() {
    let mut handler = aitch::SimpleRouter::new();
    handler.register_handler("/handler1", &HandlerFunc(handler1));
    handler.register_handler("/handler2", &HandlerFunc(handler2));

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, handler).run().unwrap();
}
