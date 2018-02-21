#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

mod errors;

use std::net::SocketAddr;

use futures::{Future, Stream};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};

use errors::*;


pub type Request = http::Request<Vec<u8>>;
pub type Response = http::Response<Vec<u8>>;
pub type ResponseBuilder = http::response::Builder;


pub trait Responder {
    fn into_response(self) -> Response;
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

impl<Fun, Resp> Handler for Fun
where
    Fun: Fn(&mut Request, ResponseBuilder) -> Resp,
    Resp: Responder,
{
    fn handle(&self, req: &mut Request, resp: ResponseBuilder) -> Response {
        (self)(req, resp).into_response()
    }
}


pub struct Server {
    addr: SocketAddr,
    handler: &'static Handler,
}

//     server.run().unwrap();

impl Server {
    pub fn new(addr: SocketAddr, handler: &'static Handler) -> Server {
        Server { addr, handler }
    }

    pub fn run(self) -> Result<()> {
        let addr = self.addr;
        hyper::server::Http::new()
            .bind(&addr, move || {
                Ok(Service {
                    handler: self.handler,
                })
            })?
            .run()?;
        Ok(())
    }
}

struct Service {
    handler: &'static Handler,
}

impl hyper::server::Service for Service {
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: HyperRequest) -> Self::Future {
        let handler = self.handler;
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
