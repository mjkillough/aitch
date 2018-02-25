#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

pub mod errors;
mod handler;
mod router;
mod server;
mod traits;

pub use handler::{AsyncHandler, AsyncHandlerFunc, Handler, SyncHandler, SyncHandlerFunc};
pub use router::SimpleRouter;
pub use server::Server;
pub use traits::{FromHttpResponse, HttpBody, IntoResponse};

use futures::Future;

pub type ResponseBuilder = http::response::Builder;
pub type FutureResponse<Body> = Box<Future<Item = http::Response<Body>, Error = ()>>;
pub type AsyncBody = hyper::Body;
pub type SyncBody = Vec<u8>;
