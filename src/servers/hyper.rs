use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, Stream};
use http;
use hyper;
use hyper::server::Server as HyperServer;

use {Body, BodyStream, Error, Handler, Responder, Result};

pub struct Server<H, ReqBody>
where
    H: Handler<ReqBody>,
    ReqBody: Body,
{
    addr: SocketAddr,
    handler: Arc<H>,
    marker: PhantomData<ReqBody>,
}

impl<H, ReqBody> Server<H, ReqBody>
where
    H: Handler<ReqBody>,
    ReqBody: Body,
{
    pub fn new(addr: SocketAddr, handler: H) -> Server<H, ReqBody> {
        let handler = Arc::new(handler);
        let marker = PhantomData;
        Server {
            addr,
            handler,
            marker,
        }
    }

    pub fn run(self) -> Result<()> {
        let handler = self.handler;
        let new_service = move || {
            let handler = handler.clone();
            hyper::service::service_fn(move |req| {
                let handler = handler.clone();
                let builder = http::Response::builder();

                map_request_body(req)
                    .and_then(move |req| handler.handle(req, builder).into_response())
                    .map(map_response_body)
                    .or_else(|err| internal_server_error(&err))
            })
        };

        let server = HyperServer::bind(&self.addr).serve(new_service);
        hyper::rt::run(server.map_err(|e| {
            eprintln!("server error: {}", e);
        }));

        Ok(())
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
    ReqBody::from_stream(Box::new(body_stream))
        .map(move |body| http::Request::from_parts(parts, body))
}

fn internal_server_error(err: &Error) -> Result<http::Response<hyper::Body>> {
    eprintln!("server error: {}", err);

    let resp = http::Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(hyper::Body::empty());
    Ok(resp?)
}

fn map_response_body(resp: http::Response<BodyStream>) -> http::Response<hyper::Body> {
    resp.map(hyper::Body::wrap_stream)
}
