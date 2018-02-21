#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

mod errors;

use std::net::SocketAddr;

use futures::future::{Future, FutureResult};
use hyper::header::ContentLength;
use hyper::server::{Request as HyperRequest, Response as HyperResponse};

use errors::*;


pub type Body = hyper::Body;
pub type Request = http::Request<hyper::Body>;
pub type Response = http::Response<&'static str>;
pub type ResponseBuilder = http::response::Builder;

pub type Handler = Fn(&mut Request, ResponseBuilder) -> Response;


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
        let builder = http::Response::builder();
        let response = (self.handler)(&mut req.into(), builder);
        let hyper_response = response.map(|body| hyper::Body::from(hyper::Chunk::from(body)));
        Box::new(futures::future::ok(hyper_response.into()))
    }
}
