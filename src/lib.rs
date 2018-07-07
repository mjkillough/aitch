extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;

mod body;
mod handler;
mod responder;
mod router;
mod server;

use std::error::Error as StdError;
use std::sync::Arc;

use bytes::Bytes;
use futures::{Future, Stream};

pub use body::{empty_body, Body, BodyStream};
pub use handler::Handler;
pub use responder::Responder;
pub use router::SimpleRouter;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

pub type Error = Box<StdError + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

type BoxedResponse = Box<Future<Item = http::Response<BodyStream>, Error = Error> + Send>;
type BoxedHandler = Box<Handler<BodyStream, Resp = BoxedResponse>>;

pub fn logging_handler<B: Body>(handler: impl Handler<B>) -> impl Handler<B> {
    move |req: http::Request<B>, resp: ResponseBuilder| {
        let method = req.method().clone();
        let uri = req.uri().clone();

        handler.handle(req, resp).into_response().map(move |resp| {
            println!("{} {} {}", method, uri.path(), resp.status());
            resp
        })
    }
}

fn map_request_body<B1, B2>(
    req: http::Request<B1>,
) -> impl Future<Item = http::Request<B2>, Error = Error>
where
    B1: Body,
    B2: Body,
{
    let (parts, body) = req.into_parts();
    B2::from_stream(body.into_stream()).map(move |body| http::Request::from_parts(parts, body))
}

pub fn box_handler<B: Body>(handler: impl Handler<B>) -> BoxedHandler {
    let handler = Arc::new(handler);
    let closure = move |req, resp| -> BoxedResponse {
        let handler = handler.clone();
        let resp =
            map_request_body(req).and_then(move |req| handler.handle(req, resp).into_response());
        Box::new(resp)
    };
    Box::new(closure)
}

pub fn response_with_status(status: http::StatusCode) -> impl Responder {
    http::Response::builder().status(status).body(empty_body())
}
