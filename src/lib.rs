extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;

mod body;
mod handler;
mod responder;
// mod router;
mod server;

use std::error::Error as StdError;
use std::sync::Arc;

use bytes::Bytes;
use futures::{Future, Stream};

pub use body::{Body, BodyStream};
pub use handler::Handler;
// pub use router::SimpleRouter;
pub use responder::Responder;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

pub type Error = Box<StdError + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

type BoxedResponse = Box<Future<Item = http::Response<BodyStream>, Error = Error> + Send>;
