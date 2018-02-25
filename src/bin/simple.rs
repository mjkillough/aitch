extern crate aitch;
extern crate futures;
extern crate http;

use aitch::{AsyncBody, AsyncHandlerFunc, FutureResponse, ResponseBuilder, SyncBody,
            SyncHandlerFunc};
use http::{Request, Response};


type SyncRequest = Request<SyncBody>;
type SyncResponse = Response<SyncBody>;


fn handler1(_req: &mut SyncRequest, mut resp: ResponseBuilder) -> http::Result<SyncResponse> {
    resp.body("Hello 1!".as_bytes().to_owned())
}

fn handler2(_req: &mut SyncRequest, mut resp: ResponseBuilder) -> SyncResponse {
    resp.body("Hello 2!".as_bytes().to_owned()).unwrap()
}

fn handler3(req: &mut Request<AsyncBody>, mut resp: ResponseBuilder) -> FutureResponse<AsyncBody> {
    let v = "Hello from the future!".as_bytes().to_owned();
    let b = AsyncBody::from(v);
    let r = resp.body(b).unwrap();
    let fut = futures::future::ok(r);
    Box::new(fut)
}

fn main() {
    let mut handler = aitch::SimpleRouter::new();
    // handler.register_handler("/handler1", AsyncHandlerFunc(handler3));
    handler.register_handler("/handler2", SyncHandlerFunc(handler2));
    handler.register_handler(
        "/handler3",
        SyncHandlerFunc(|req, resp| handler2(req, resp)),
    );

    let addr = "127.0.0.1:3000".parse().unwrap();
    aitch::Server::new(addr, handler).run().unwrap();
}
