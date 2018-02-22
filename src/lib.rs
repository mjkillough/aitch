#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

mod errors;

use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, Stream};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};

use errors::*;


pub type Request = http::Request<Vec<u8>>;
pub type Response = http::Response<Vec<u8>>;
pub type ResponseBuilder = http::response::Builder;


pub trait Responder {
    fn into_response(self) -> Response;
}

impl Responder for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl Responder for http::Result<Response> {
    fn into_response(self) -> Response {
        self.unwrap_or_else(|_| {
            http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Vec::new())
                .unwrap()
        })
    }
}


pub trait Handler {
    fn handle(&self, &mut Request, ResponseBuilder) -> Response;
}


pub struct HandlerFunc<Func, Resp>(pub Func)
where
    Func: Fn(&mut Request, ResponseBuilder) -> Resp,
    Resp: Responder;

impl<Func, Resp> Handler for HandlerFunc<Func, Resp>
where
    Func: Fn(&mut Request, ResponseBuilder) -> Resp,
    Resp: Responder,
{
    fn handle(&self, req: &mut Request, resp: ResponseBuilder) -> Response {
        (self.0)(req, resp).into_response()
    }
}


pub struct Server<H>
where
    H: Handler + 'static,
{
    addr: SocketAddr,
    handler: Arc<H>,
}

impl<H> Server<H>
where
    H: Handler + 'static,
{
    pub fn new(addr: SocketAddr, handler: H) -> Server<H> {
        let handler = Arc::new(handler);
        Server { addr, handler }
    }

    pub fn run(self) -> Result<()> {
        let addr = self.addr;
        hyper::server::Http::new()
            .bind(&addr, move || {
                Ok(Service {
                    handler: self.handler.clone(),
                })
            })?
            .run()?;
        Ok(())
    }
}

struct Service<H>
where
    H: Handler + 'static,
{
    handler: Arc<H>,
}

impl<H> hyper::server::Service for Service<H>
where
    H: Handler + 'static,
{
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: HyperRequest) -> Self::Future {
        let handler = self.handler.clone();
        let req: http::Request<hyper::Body> = req.into();
        let (parts, body) = req.into_parts();
        let fut = body.concat2().and_then(move |body| {
            let req = Request::from_parts(parts, body.to_vec());
            let builder = http::Response::builder();
            let response = handler.handle(&mut req.into(), builder);
            let hyper_response = response.map(|body| hyper::Body::from(hyper::Chunk::from(body)));
            futures::future::ok(hyper_response.into())
        });
        Box::new(fut)
    }
}
