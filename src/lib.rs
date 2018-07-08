#![deny(warnings)]

extern crate bytes;
extern crate futures;
extern crate http;

#[cfg(feature = "hyper")]
extern crate hyper;

#[cfg(feature = "json")]
extern crate serde;
#[cfg(feature = "json")]
extern crate serde_json;

#[cfg(feature = "tiny_http")]
extern crate tiny_http;

mod body;
mod handler;
#[cfg(feature = "json")]
mod json;
pub mod middlewares;
mod responder;
pub mod servers;

use std::error::Error as StdError;

use futures::Future;

pub use body::{empty_body, Body, BodyStream};
pub use handler::{box_handler, BoxedHandler, Handler};
pub use responder::Responder;

#[cfg(feature = "json")]
pub use json::Json;

pub type ResponseBuilder = http::response::Builder;

pub type Error = Box<StdError + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

type BoxedResponse = Box<Future<Item = http::Response<BodyStream>, Error = Error> + Send>;

pub fn response_with_status(status: http::StatusCode) -> impl Responder {
    http::Response::builder().status(status).body(empty_body())
}
