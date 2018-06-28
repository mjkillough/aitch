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

use futures::{Future, IntoFuture};

pub type ResponseBuilder = http::response::Builder;

type BoxedResponse<Body> = Box<Future<Item = http::Response<Body>, Error = http::Error> + 'static>;

pub fn box_response<H, Body, Resp>(handler: H) -> impl Handler<Body, BoxedResponse<Body>>
where
    H: Handler<Body, Resp>,
    Body: traits::HttpBody,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + 'static,
{
    move |req: &mut http::Request<Body>, resp: ResponseBuilder| {
        Box::new(handler.handle(req, resp).into_future()) as BoxedResponse<Body>
    }
}
