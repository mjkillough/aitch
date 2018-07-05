use std::error::Error as StdError;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use futures::{Future, Stream};
use http;
use hyper;
use hyper::server::Server as HyperServer;

use errors::*;
use {Body, Handler, Responder};

pub struct Server<H, ReqBody, Resp, RespBody>
where
    H: Handler<ReqBody, Resp, RespBody>,
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
{
    addr: SocketAddr,
    handler: Arc<H>,
    marker: PhantomData<(ReqBody, Resp, RespBody)>,
}

impl<H, ReqBody, Resp, RespBody> Server<H, ReqBody, Resp, RespBody>
where
    H: Handler<ReqBody, Resp, RespBody>,
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
{
    pub fn new(addr: SocketAddr, handler: H) -> Server<H, ReqBody, Resp, RespBody> {
        let handler = Arc::new(handler);
        let marker = PhantomData;
        Server {
            addr,
            handler,
            marker,
        }
    }

    pub fn run(self) {
        let addr = self.addr;
        let new_service = move || -> Result<Service<H, ReqBody, Resp, RespBody>> {
            Ok(Service {
                handler: self.handler.clone(),
                marker: PhantomData,
            })
        };
        let server = HyperServer::bind(&addr).serve(new_service);

        hyper::rt::run(server.map_err(|e| {
            eprintln!("server error: {}", e);
        }));
    }
}

struct Service<H, ReqBody, Resp, RespBody>
where
    H: Handler<ReqBody, Resp, RespBody>,
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
{
    handler: Arc<H>,
    marker: PhantomData<(ReqBody, Resp, RespBody)>,
}

impl<H, ReqBody, Resp, RespBody> hyper::service::Service for Service<H, ReqBody, Resp, RespBody>
where
    H: Handler<ReqBody, Resp, RespBody>,
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = Error;
    type Future = Box<Future<Item = hyper::Response<hyper::Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        let handler = self.handler.clone();
        let req: http::Request<hyper::Body> = req.into();
        let (parts, body) = req.into_parts();
        let body_stream = body.map(hyper::Chunk::into_bytes).map_err(Error::from);
        let fut = Body::from_stream(body_stream)
            .and_then(move |body| {
                let mut req = http::Request::from_parts(parts, body);
                let builder = http::Response::builder();
                handler
                    .handle(&mut req, builder)
                    .into_response()
                    .map_err(Error::from)
            })
            .map(map_response_body)
            .or_else(|_| {
                http::Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(hyper::Body::empty())
                    .map_err(Error::from)
            });
        Box::new(fut)
    }
}

type HyperBodyStream =
    Box<Stream<Item = hyper::Chunk, Error = Box<StdError + Send + Sync>> + Send + 'static>;

fn map_response_body(
    resp: http::Response<Box<Stream<Item = Bytes, Error = http::Error> + Send>>,
) -> http::Response<hyper::Body> {
    let (parts, body) = resp.into_parts();
    let stream = body.map(|bytes| hyper::Chunk::from(bytes))
        .map_err(|error| Box::new(error) as Box<StdError + Send + Sync>);
    let hyper_body = hyper::Body::from(Box::new(stream) as HyperBodyStream);
    http::Response::from_parts(parts, hyper_body)
}
