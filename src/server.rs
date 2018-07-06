use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, Stream};
use http;
use hyper;
use hyper::server::Server as HyperServer;

use {Body, BoxedStream, Error, Handler, Responder, Result};

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
        let builder = http::Response::builder();

        let fut = map_request_body(req)
            .and_then(move |mut req| handler.handle(&mut req, builder).into_response())
            .map(map_response_body)
            .or_else(|_| internal_server_error());

        Box::new(fut)
    }
}

fn map_request_body<ReqBody>(
    req: http::Request<hyper::Body>,
) -> impl Future<Item = http::Request<ReqBody>, Error = Error>
where
    ReqBody: Body,
{
    let (parts, body) = req.into_parts();
    let body_stream = body.map(hyper::Chunk::into_bytes).map_err(Box::from);
    Body::from_stream(body_stream).map(move |body| http::Request::from_parts(parts, body))
}

fn internal_server_error() -> Result<http::Response<hyper::Body>> {
    let resp = http::Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(hyper::Body::empty());
    Ok(resp?)
}

fn map_response_body(resp: http::Response<BoxedStream>) -> http::Response<hyper::Body> {
    resp.map(hyper::Body::wrap_stream)
}
