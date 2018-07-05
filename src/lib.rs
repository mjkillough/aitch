extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;

pub mod errors;
// mod router;
mod server;

use bytes::Bytes;
use futures::{stream, Future, IntoFuture, Stream};

use errors::Error;

// pub use router::SimpleRouter;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

type BoxedBody = Box<Stream<Item = Bytes, Error = http::Error> + Send>;
type BoxedResponse = Box<Future<Item = http::Response<BoxedBody>, Error = http::Error> + Send>;

pub trait Body
where
    Self: Send + 'static,
{
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static;

    fn into_stream(self) -> BoxedBody;
}

impl Body for Vec<u8> {
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static,
    {
        Box::new(stream.concat2().map(|bytes| bytes.to_vec()))
    }

    fn into_stream(self) -> BoxedBody {
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
    T: IntoFuture<Item = http::Response<B>, Error = http::Error> + Send + 'static,
    T::Future: Send + 'static,
    B: Body,
{
    fn into_response(self) -> BoxedResponse {
        let fut = self.into_future().map(|resp| {
            let (parts, body) = resp.into_parts();
            http::Response::from_parts(parts, body.into_stream())
        });
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
