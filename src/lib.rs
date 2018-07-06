extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;

mod body;
mod handler;
// mod router;
mod responder;
mod server;

use std::error::Error as StdError;

use bytes::Bytes;
use futures::{Future, Stream};

pub use body::Body;
pub use handler::Handler;
// pub use router::SimpleRouter;
pub use responder::Responder;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

pub type Error = Box<StdError + Send + Sync>;
pub type Result<T> = ::std::result::Result<T, Error>;

type BoxedStream = Box<Stream<Item = Bytes, Error = Error> + Send>;
type BoxedResponse = Box<Future<Item = http::Response<BoxedStream>, Error = Error> + Send>;
