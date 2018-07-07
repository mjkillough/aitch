#![deny(warnings)]

extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;

mod body;
mod handler;
pub mod middlewares;
mod responder;
mod server;

use std::error::Error as StdError;

use futures::Future;

pub use body::{empty_body, Body, BodyStream};
pub use handler::{box_handler, BoxedHandler, Handler};
pub use responder::Responder;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

pub type Error = Box<StdError + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

type BoxedResponse = Box<Future<Item = http::Response<BodyStream>, Error = Error> + Send>;

pub fn response_with_status(status: http::StatusCode) -> impl Responder {
    http::Response::builder().status(status).body(empty_body())
}
