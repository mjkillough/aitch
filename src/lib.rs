#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

pub mod errors;
mod handler;
mod router;
mod server;
pub mod traits;

pub use handler::Handler;
pub use router::SimpleRouter;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;

pub mod sync {
    pub type SyncBody = Vec<u8>;

    pub use handler::{SyncHandler, SyncHandlerFunc};
}

pub mod async {
    use futures::Future;
    use hyper;
    use http;

    pub type AsyncBody = hyper::Body;
    pub type FutureResponse<Body> = Box<Future<Item = http::Response<Body>, Error = ()>>;

    pub use handler::{AsyncHandler, AsyncHandlerFunc};
}
