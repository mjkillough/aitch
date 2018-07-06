extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;

mod body;
mod handler;
// mod router;
mod responder;
mod server;

use std::error::Error as StdError;
use std::sync::Arc;

use bytes::Bytes;
use futures::{Future, Stream};

pub use body::{Body, BodyStream};
pub use handler::Handler;
// pub use router::SimpleRouter;
pub use responder::Responder;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

pub type Error = Box<StdError + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

type BoxedResponse = Box<Future<Item = http::Response<BodyStream>, Error = Error> + Send>;

fn logging_handler<H, ReqBody, Resp, RespBody>(handler: H) -> impl Handler<ReqBody, Resp, RespBody>
where
    H: Handler<ReqBody, Resp, RespBody>,
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
{
    move |req: http::Request<ReqBody>, mut resp: ResponseBuilder| {
        println!("Request");
        handler.handle(req, resp)
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

fn genericify<H, ReqBody, Resp, RespBody>(
    handler: H,
) -> impl Handler<BodyStream, BoxedResponse, BodyStream>
where
    H: Handler<ReqBody, Resp, RespBody>,
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
{
    let handler: Arc<H> = Arc::new(handler);
    move |req, mut resp| {
        let handler = handler.clone();
        println!("Request");
        map_request_body(req)
            .and_then(move |req| handler.clone().handle(req, resp).into_response())
            .into_response()
    }
}
