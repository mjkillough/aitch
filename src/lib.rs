#![deny(warnings)]

//! aitch is a simple, lightweight toolkit for building HTTP servers in safe, stable Rust.
//!
//! It builds upon the [`http` crate], and provides additional types for representing HTTP handlers,
//! bodies and middlewares. It provides both [`hyper`] and [`tiny_http`] backends for running
//! handlers, but aims to be agnostic of the HTTP server library.
//!
//! It's inspired by the simplicity (and popularity) of Go's [`net/http` package], which builds
//! applications/middlewares as a series of nested [`Handler`s].
//!
//! [`http` crate]: https://github.com/hyperium/http
//! [`hyper`]: https://hyper.rs/
//! [`tiny_http`]: https://github.com/tiny-http/tiny-http
//! [`net/http` package]: https://golang.org/pkg/net/http/
//! [`Handler`s]: https://golang.org/pkg/net/http/#Handler
//!
//! # Defining Handler functions
//!
//! Handlers are typically defined as functions with the signature:
//!
//! ```ignore
//! Fn(http::Request<impl Body>, http::response::Builder) -> impl Responder
//! ```
//!
//! Both synchronous and asychronous handlers are defined using this same signature.
//!
//! A [`Responder`] is anything that implements `IntoFuture<Item = http::Response<impl Body>>`,
//! whose `Error` can be converted into a `Box<Error>`. This will often be a [`Result`] for
//! synchronous handlers, or something implementing [`Future`] for asynchronous handlers.
//!
//! For example, to define a simple sychronous handler as a function:
//!
//! ```
//! # extern crate aitch;
//! # extern crate http;
//! #
//! # use aitch::Responder;
//! #
//! fn handler(_: http::Request<()>, mut resp: http::response::Builder) -> impl Responder {
//!     resp.body("Hello, world!".to_owned())
//! }
//! ```
//!
//! [`Responder`]: trait.Responder.html
//! [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
//! [`Future`]: https://docs.rs/futures/0.1.23/futures/future/trait.Future.html
//!
//! ### More complex Handlers using the `Handler` trait
//!
//! Handlers aren't limited to functions - any type that implements the [`Handler`] trait can be used.
//!
//! For an example of how to implement the [`Handler`] trait for a more complex type, see the
//! provided [`SimpleRouter`], which routes HTTP requests to one of many handlers, based on the path
//! in the HTTP request.
//!
//! [`Handler`]: trait.Hander.html
//! [`SimpleRouter`]: struct.SimpleRouter.html
//!
//! # Serving Requests
//!
//! In order for a `Handler` to serve requests, it needs to be passed to a server which can process
//! HTTP requests.
//!
//! aitch comes with two server implementations:
//!
//!  - [`aitch::servers::hyper::Server`], which uses [`hyper`] to receive and process requests. As
//!    [`hyper`] can take full advantage of asynchronous I/O, this server backend can support
//!    streaming request/response bodies.
//!  - [`aitch::servers::tiny_http::Server`], which uses [`tiny_http`] to receive and process
//!    requests. [`tiny_http`] uses a thread-pool to synchronously process each request, and the
//!    entire request body will be buffered before being past to the Handler - there is no support
//!    for streaming bodies.
//!
//! The following example demonstrates passing a Handler function to
//! [`aitch::servers::hyper::Server`], which allows it to listen and respond to requests:
//!
//! ```no_run
//! # extern crate aitch;
//! # extern crate http;
//! #
//! # use aitch::servers::hyper::Server;
//! # use aitch::{middlewares, Responder, ResponseBuilder, Result};
//! # use http::Request;
//! #
//! # fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
//! #    resp.body("Hello, world!".to_owned())
//! # }
//! #
//! # fn main() -> Result<()> {
//! let addr = "127.0.0.1:3000".parse()?;
//! aitch::servers::hyper::Server::new(addr, handler).run()
//! # }
//! ```
//!
//! aitch aims to be agnostic of the server technology, and it should be possible to add support for
//! other servers in third-party crates. The source code of the two provided server implementations
//! should demonstrate how to do this.
//!
//! [`aitch::servers::hyper::Server`]: servers/hyper/struct.Server.html
//! [`aitch::servers::tiny_http::Server`]: servers/tiny_http/struct.Server.html
//!
//! # Dealing with Request/Response Bodies
//!
//! // TODO
//!
//! # Writing Middlewares
//!
//! // TODO
//!

extern crate bytes;
extern crate futures;
extern crate http;

#[cfg(feature = "json")]
extern crate serde;
#[cfg(feature = "json")]
extern crate serde_json;

#[cfg(feature = "mime_guess")]
extern crate mime_guess;

#[cfg(feature = "server-hyper")]
extern crate hyper;

#[cfg(feature = "server-tiny-http")]
extern crate tiny_http;
#[cfg(feature = "server-tiny-http")]
extern crate tokio_threadpool;

mod body;
mod handler;
pub mod handlers;
#[cfg(feature = "json")]
mod json;
pub mod middlewares;
mod responder;
pub mod servers;

use std::error::Error as StdError;

use futures::Future;

pub use body::{Body, BodyStream};
pub use handler::{box_handler, BoxedHandler, Handler};
pub use responder::Responder;

#[cfg(feature = "json")]
pub use json::Json;

/// A type alias for [`http::response::Builder`].
///
/// This allows a simpler type signature for handler functions.
///
/// # Example
///
/// ```
/// # extern crate aitch;
/// # extern crate http;
/// #
/// # use aitch::{ResponseBuilder, Responder};
/// # use http::Request;
/// #
/// fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
///     resp.body("Hello, world".to_owned())
/// }
/// ```
///
/// [`http::response::Builder`]: https://docs.rs/http/0.1.7/http/response/struct.Builder.html
pub type ResponseBuilder = http::response::Builder;

/// A generic error type for aitch handlers and middlewares.
///
/// An error type of `Box<std::error::Error>` is used, so that the error type is as generic as possible
/// and can be passed through layers of third-party handlers and middlewares.
///
/// In most cases, handlers and middlewares should aim to handle their own errors (and return an
/// appropriate HTTP response). Returning an error from a handler should be an exceptional
/// circumstance, and will most likely (depending on the middleware/server in use) result in a
/// generic HTTP 500 error page.
pub type Error = Box<StdError + Send + Sync>;

/// A type alias to make working with `Result<T, aitch::Error>` more convenient.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Represents a future returning a HTTP response, with all types erased.
///
/// Handlers which return one of many different [`Responder`] types (e.g. depending on the HTTP
/// request details) can use this type to return a generic response, with all type variables erased.
///
/// To get a `BoxedResponse`, use the [`Responder::into_response()`] trait method.
///
/// [`Responder`]: trait.Responder.html
/// [`Responder::into_response()`]: trait.Responder.html#tymethod.into_response
///
/// # Example
///
/// See [`BoxedHandler`] for an example of its use.
///
/// [`BoxedHandler`]: type.BoxedHandler.html
pub type BoxedResponse = Box<Future<Item = http::Response<BodyStream>, Error = Error> + Send>;
