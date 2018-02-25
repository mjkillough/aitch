#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

pub mod errors;
mod handler;
mod router;
mod traits;

pub use handler::{AsyncHandler, AsyncHandlerFunc, Handler, SyncHandler, SyncHandlerFunc};
pub use router::SimpleRouter;
pub use traits::{EmptyBody, FromHttpResponse, IntoResponse};

use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, Stream};

use errors::*;


pub type ResponseBuilder = http::response::Builder;
pub type FutureResponse<Body> = Box<Future<Item = http::Response<Body>, Error = ()>>;
pub type AsyncBody = hyper::Body;
pub type SyncBody = Vec<u8>;


pub struct Server<H, Body, Resp>
where
    H: Handler<Body, Resp> + 'static,
    Body: EmptyBody + 'static,
    Resp: FromHttpResponse<Body> + 'static,
{
    addr: SocketAddr,
    handler: Arc<H>,
    marker: PhantomData<(Body, Resp)>,
}

impl<H, Body, Resp> Server<H, Body, Resp>
where
    H: Handler<Body, Resp> + 'static,
    Body: EmptyBody + 'static,
    Resp: FromHttpResponse<Body> + 'static,
{
    pub fn new(addr: SocketAddr, handler: H) -> Server<H, Body, Resp> {
        let handler = Arc::new(handler);
        let marker = PhantomData;
        Server {
            addr,
            handler,
            marker,
        }
    }
}

impl<H> Server<H, AsyncBody, FutureResponse<AsyncBody>>
where
    H: AsyncHandler + 'static,
{
    pub fn run(self) -> Result<()> {
        let addr = self.addr;
        hyper::server::Http::new()
            .bind(&addr, move || {
                Ok(Service {
                    handler: self.handler.clone(),
                    marker: PhantomData,
                })
            })?
            .run()?;
        Ok(())
    }
}

impl<H> Server<H, Vec<u8>, http::Response<Vec<u8>>>
where
    H: SyncHandler + 'static,
{
    pub fn run(self) -> Result<()> {
        let addr = self.addr;
        hyper::server::Http::new()
            .bind(&addr, move || {
                Ok(Service {
                    handler: self.handler.clone(),
                    marker: PhantomData,
                })
            })?
            .run()?;
        Ok(())
    }
}


struct Service<H, Body, Resp>
where
    H: Handler<Body, Resp> + 'static,
    Body: EmptyBody + 'static,
    Resp: FromHttpResponse<Body> + 'static,
{
    handler: Arc<H>,
    marker: PhantomData<(Body, Resp)>,
}

impl<H> hyper::server::Service for Service<H, AsyncBody, FutureResponse<AsyncBody>>
where
    H: AsyncHandler + 'static,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let handler = self.handler.clone();
        let req: http::Request<hyper::Body> = req.into();
        let builder = http::Response::builder();
        let fut = handler
            .handle(&mut req.into(), builder)
            .or_else(|_| {
                FutureResponse::<AsyncBody>::from_http_response(
                    ResponseBuilder::new()
                        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                        .body(AsyncBody::empty())
                        .unwrap(),
                )
            })
            .map(hyper::Response::from)
            // This should never happen. There isn't really a more sensible
            // `hyper::Error` to return either.
            .map_err(|_| hyper::Error::Timeout);
        Box::new(fut)
    }
}


impl<H> hyper::server::Service for Service<H, Vec<u8>, http::Response<Vec<u8>>>
where
    H: SyncHandler + 'static,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let handler = self.handler.clone();
        let req: http::Request<hyper::Body> = req.into();
        let (parts, body) = req.into_parts();
        let fut = body.concat2().and_then(move |body| {
            let req = http::Request::<Vec<u8>>::from_parts(parts, body.to_vec());
            let builder = http::Response::builder();
            let response = handler.handle(&mut req.into(), builder);
            let hyper_response = response.map(|body| hyper::Body::from(hyper::Chunk::from(body)));
            futures::future::ok(hyper_response.into())
        });
        Box::new(fut)
    }
}
