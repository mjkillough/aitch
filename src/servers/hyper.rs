//! Provides a [`hyper`] server, which can serve a handler.
//!
//! This module provides [`Server`], which is a [`hyper`] server which uses a [`Handler`] to respond
//! to incoming HTTP requests.
//!
//! See the documentation of [`Server`] for more detail.
//!
//! [`hyper`]: https://hyper.rs
//! [`Server`]: struct.Server.html
//! [`Handler`]: ../../trait.Handler.html

use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, Stream};
use http;
use hyper;
use hyper::server::Server as HyperServer;

use {Body, BodyStream, Error, Handler, Responder, Result};

// Works around lack of Box<FnOnce>/FnBox.
pub trait ServeFunc {
    fn call_box(self: Box<Self>) -> Result<()>;
}

impl<F> ServeFunc for F
where
    F: FnOnce() -> Result<()>,
{
    fn call_box(self: Box<Self>) -> Result<()> {
        (*self)()
    }
}

/// A [`hyper`] server, which can serve a handler.
///
/// This server back-end uses [`hyper`] to listen for incoming HTTP requests, call the provided
/// [`Handler`], and then return the response to the client.
///
/// The server uses the default [`tokio`] runtime to serve requests, which uses a separate reactor
/// to drive I/O resources, and a thread-pool which uses those I/O resources to construct HTTP
/// requests/responses. The provided handler (and any handlers it may call) are all run on this
/// thread-pool.
///
/// [`hyper`]: https://hyper.rs
/// [`Handler`]: ../../trait.Handler.html
/// [`tokio`]: https://tokio.rs
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
/// aitch::servers::hyper::Server::new(addr, handler)?.run()
/// # }
/// ```
pub struct Server<H, ReqBody>
where
    H: Handler<ReqBody>,
    ReqBody: Body,
{
    addr: SocketAddr,
    serve: Box<ServeFunc + Send>,
    marker: PhantomData<(H, ReqBody)>,
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
    pub fn new(addr: SocketAddr, handler: H) -> Result<Server<H, ReqBody>> {
        let (addr, serve) = Server::construct_server(addr, handler);
        let marker = PhantomData;
        Ok(Server {
            addr,
            serve,
            marker,
        })
    }

    fn construct_server(addr: SocketAddr, handler: H) -> (SocketAddr, Box<ServeFunc + Send>) {
        let handler = Arc::new(handler);
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

        let server = HyperServer::bind(&addr).serve(new_service);
        let addr = server.local_addr();

        let closure = move || {
            hyper::rt::run(server.map_err(|e| {
                eprintln!("server error: {}", e);
            }));

            Ok(())
        };

        (addr, Box::new(closure))
    }

    /// Returns the address that the server is listening on.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Starts and runs the server.
    pub fn run(self) -> Result<()> {
        self.serve.call_box()
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
