use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, IntoFuture, Stream};
use http;
use hyper;

use errors::*;
use traits::HttpBody;
use Handler;

pub struct Server<H, Body, Resp>
where
    H: Handler<Body, Resp> + 'static,
    Body: HttpBody + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + 'static,
{
    addr: SocketAddr,
    handler: Arc<H>,
    marker: PhantomData<(Body, Resp)>,
}

impl<H, Body, Resp> Server<H, Body, Resp>
where
    H: Handler<Body, Resp> + 'static,
    Body: HttpBody + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + 'static,
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
    Body: HttpBody + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + 'static,
{
    handler: Arc<H>,
    marker: PhantomData<(Body, Resp)>,
}

impl<H, Body, Resp> hyper::server::Service for Service<H, Body, Resp>
where
    H: Handler<Body, Resp> + 'static,
    Body: HttpBody + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + 'static,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: hyper::Request) -> Self::Future {
        let handler = self.handler.clone();
        let req: http::Request<hyper::Body> = req.into();
        let (parts, body) = req.into_parts();
        let fut = Body::from_hyper_body(body)
            .and_then(move |body| {
                let mut req = http::Request::from_parts(parts, body);
                let builder = http::Response::builder();
                handler
                    .handle(&mut req, builder)
                    .into_future()
                    .map_err(Error::from)
            })
            .or_else(|_| {
                http::Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .map_err(Error::from)
            })
            .and_then(map_response_body)
            .map(hyper::Response::from)
            // This should never happen. There isn't really a more sensible
            // `hyper::Error` to return either.
            .map_err(|_| hyper::Error::Timeout);
        Box::new(fut)
    }
}

fn map_response_body<Body>(
    resp: http::Response<Body>,
) -> impl Future<Item = http::Response<hyper::Body>, Error = Error>
where
    Body: HttpBody,
{
    let (parts, body) = resp.into_parts();
    body.into_stream().concat2().map(move |buffer| {
        let chunk = hyper::Chunk::from(buffer);
        let body = hyper::Body::from(chunk);
        http::Response::from_parts(parts, body)
    })
}
