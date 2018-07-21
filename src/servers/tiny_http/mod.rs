//! Provides a [`tiny_http`] server, which can serve a handler.
//!
//! This module provides [`Server`], which is a [`tiny_http`] server which uses a [`Handler`] to respond
//! to incoming HTTP requests.
//!
//! See the documentation of [`Server`] for more detail.
//!
//! [`tiny_http`]: https://github.com/tiny-http/tiny-http
//! [`Server`]: struct.Server.html
//! [`Handler`]: ../../trait.Handler.html

mod request;
mod response;

use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{future, Future};
use http;
use tiny_http;
use tokio_threadpool::ThreadPool;

use self::request::as_http_request;
use self::response::as_tiny_http_response;
use {Body, Handler, Responder, Result};

/// A [`tiny_http`] server, which can serve a handler.
///
/// This server back-end uses [`tiny_http`] to listen for incoming HTTP requests, call the provided
/// [`Handler`], and then return the response to the client.
///
/// The server uses the internal thread-pool of [`tiny_http`] to construct HTTP requests/responses
/// from the underyling connection. The [`Handler`] (and any handlers it may call) are spawned onto
/// a thread-pool managed by [`tokio-threadpool`], and so should not block requests from being
/// processed.
///
/// This server is unable to process streaming HTTP request or response bodies. The full request
/// body will be buffered in memory before the provided [`Handler`] is called. The full response
/// body will be read from the returned [`Responder`] before being sent to the client.
///
/// Users who wish to write asynchronous handlers, are encouraged to instead use the [`hyper`
/// back-end].
///
/// [`tiny_http`]: https://github.com/tiny-http/tiny-http
/// [`Handler`]: ../../trait.Handler.html
/// [`Responder`]: ../../trait.Responder.html
/// [`tokio-threadpool`]: https://crates.io/crates/tokio-threadpool
/// [`hyper` back-end]: ../hyper/struct.Server.html
///
/// # Example
///
/// ```no_run
/// # extern crate aitch;
/// # extern crate http;
/// #
/// # use aitch::{middlewares, Responder, ResponseBuilder, Result};
/// # use http::Request;
/// #
/// # fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
/// #    resp.body("Hello, world!".to_owned())
/// # }
/// #
/// # fn main() -> Result<()> {
/// let addr = "127.0.0.1:3000".parse()?;
/// aitch::servers::tiny_http::Server::new(addr, handler).run()
/// # }
/// ```
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
    /// Creates a server which will listen on the provided [`SocketAddr`] and handle requests using
    /// the provided [`Handler`].
    ///
    /// [`SocketAddr`]: https://doc.rust-lang.org/std/net/enum.SocketAddr.html
    /// [`Handler`]: ../../trait.Handler.html
    pub fn new(addr: SocketAddr, handler: H) -> Server<H, ReqBody> {
        let handler = Arc::new(handler);
        let marker = PhantomData;
        Server {
            addr,
            handler,
            marker,
        }
    }

    /// Starts and runs the server.
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
            .and_then(as_tiny_http_response)
            .and_then(move |resp| {
                req.respond(resp)?;
                Ok(())
            })
            .or_else(|err| {
                eprintln!("Server error processing request: {}", err);
                Ok(())
            })
    }
}
