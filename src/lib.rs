#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

pub mod errors;
mod handler;
// pub mod middleware;
// mod router;
mod server;
pub mod traits;

pub use handler::Handler;
// pub use router::SimpleRouter;
pub use server::Server;

pub type ResponseBuilder = http::response::Builder;
