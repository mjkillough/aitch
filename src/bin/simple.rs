extern crate aitch;
extern crate http;

use aitch::{Handler, HandlerFunc, Request, Response, ResponseBuilder};

fn handler1(_req: &mut Request, mut resp: ResponseBuilder) -> http::Result<Response> {
    resp.body("Hello world!".as_bytes().to_owned())
}

fn handler2(_req: &mut Request, mut resp: ResponseBuilder) -> Response {
    resp.body("Hello world!".as_bytes().to_owned()).unwrap()
}


struct Handler3;

impl Handler for Handler3 {
    fn handle(&self, _req: &mut Request, mut resp: ResponseBuilder) -> Response {
        resp.body("Hello world!".as_bytes().to_owned()).unwrap()
    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, HandlerFunc(handler1))
        .run()
        .unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, HandlerFunc(&handler1))
        .run()
        .unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, HandlerFunc(handler2))
        .run()
        .unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, HandlerFunc(&handler2))
        .run()
        .unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, HandlerFunc(|req, resp| handler2(req, resp)))
        .run()
        .unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, Handler3 {}).run().unwrap();
}
