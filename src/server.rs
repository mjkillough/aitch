use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, IntoFuture, Stream};
use http;
use hyper;
use hyper::server::Server as HyperServer;

use errors::*;
use {Body as BodyTrait, Handler};

pub struct Server<H, Body, Resp>
where
    H: Handler<Body, Resp> + Send + Sync + 'static,
    Body: BodyTrait + Send + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + Send + 'static,
    Resp::Future: Send,
{
    addr: SocketAddr,
    handler: Arc<H>,
    marker: PhantomData<(Body, Resp)>,
}

impl<H, Body, Resp> Server<H, Body, Resp>
where
    H: Handler<Body, Resp> + Send + Sync + 'static,
    Body: BodyTrait + Send + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + Send + 'static,
    Resp::Future: Send,
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

    pub fn run(self) {
        let addr = self.addr;
        let new_service = move || -> Result<Service<H, Body, Resp>> {
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

struct Service<H, Body, Resp>
where
    H: Handler<Body, Resp> + Send + Sync + 'static,
    Body: BodyTrait + Send + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + Send + 'static,
    Resp::Future: Send,
{
    handler: Arc<H>,
    marker: PhantomData<(Body, Resp)>,
}

impl<H, Body, Resp> hyper::service::Service for Service<H, Body, Resp>
where
    H: Handler<Body, Resp> + Send + Sync + 'static,
    Body: BodyTrait + Send + 'static,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + Send + 'static,
    Resp::Future: Send,
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
        let fut = BodyTrait::from_stream(body_stream)
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
            .map(hyper::Response::from);
        Box::new(fut)
    }
}

fn map_response_body<Body>(
    resp: http::Response<Body>,
) -> impl Future<Item = http::Response<hyper::Body>, Error = Error>
where
    Body: BodyTrait,
{
    let (parts, body) = resp.into_parts();
    body.into_stream().concat2().map(move |buffer| {
        let chunk = hyper::Chunk::from(buffer);
        let body = hyper::Body::from(chunk);
        http::Response::from_parts(parts, body)
    })
}
