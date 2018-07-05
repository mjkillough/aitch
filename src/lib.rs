extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;

// mod router;
mod server;

use std::error::Error as StdError;

use bytes::Bytes;
use futures::{stream, Future, IntoFuture, Stream};

// pub use router::SimpleRouter;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

pub type Error = Box<StdError + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

type BoxedStream = Box<Stream<Item = Bytes, Error = Error> + Send>;
type BoxedResponse = Box<Future<Item = http::Response<BoxedStream>, Error = Error> + Send>;

pub trait Body
where
    Self: Send + 'static,
{
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static;

    fn into_stream(self) -> BoxedStream;
}

impl Body for Vec<u8> {
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static,
    {
        Box::new(stream.concat2().map(|bytes| bytes.to_vec()))
    }

    fn into_stream(self) -> BoxedStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

pub trait Responder<B>
where
    B: Body,
    Self: Send + 'static,
{
    fn into_response(self) -> BoxedResponse;
}

impl<T, B> Responder<B> for T
where
    T: IntoFuture<Item = http::Response<B>> + Send + 'static,
    T::Error: Into<Error>,
    T::Future: Send + 'static,
    B: Body,
{
    fn into_response(self) -> BoxedResponse {
        let fut = self.into_future()
            .map(|resp| {
                let (parts, body) = resp.into_parts();
                http::Response::from_parts(parts, body.into_stream())
            })
            .map_err(|error| error.into());
        Box::new(fut) as BoxedResponse
    }
}

pub trait Handler<ReqBody, Resp, RespBody>
where
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
    Self: Send + Sync + 'static,
{
    fn handle(&self, &mut http::Request<ReqBody>, ResponseBuilder) -> Resp;
}

impl<Func, ReqBody, Resp, RespBody> Handler<ReqBody, Resp, RespBody> for Func
where
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
    Func: Fn(&mut http::Request<ReqBody>, ResponseBuilder) -> Resp + Send + Sync + 'static,
{
    fn handle(&self, req: &mut http::Request<ReqBody>, resp: ResponseBuilder) -> Resp {
        (self)(req, resp)
    }
}
