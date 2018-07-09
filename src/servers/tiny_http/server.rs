use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{future, Future};
use http;
use tiny_http;
use tokio_threadpool::ThreadPool;

use servers::tiny_http::request::as_http_request;
use servers::tiny_http::response::as_tiny_http_response;
use {Body, Handler, Responder, Result};

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
        let server = tiny_http::Server::http(self.addr)?;
        let pool = ThreadPool::new();

        loop {
            let req = match server.recv() {
                Ok(req) => req,
                Err(e) => {
                    eprintln!("Server error: {}", e);
                    continue;
                }
            };

            let handler = self.handler.clone();
            pool.spawn(future::lazy(move || Server::process_request(handler, req)));
        }
    }

    fn process_request(
        handler: Arc<H>,
        mut req: tiny_http::Request,
    ) -> impl Future<Item = (), Error = ()> {
        let http_request = as_http_request(&mut req);

        future::lazy(move || Ok(http_request?.into_parts()))
            .and_then(|(parts, body)| {
                body.into_body::<ReqBody>()
                    .map(move |body| http::Request::from_parts(parts, body))
            })
            .and_then(move |http_request| {
                handler
                    .handle(http_request, http::Response::builder())
                    .into_response()
            })
            .and_then(move |http_response| as_tiny_http_response(http_response))
            .and_then(move |resp| Ok(req.respond(resp)?))
            .or_else(|err| {
                eprintln!("Server error processing request: {}", err);
                Ok(())
            })
    }
}
