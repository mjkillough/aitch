extern crate aitch;
extern crate futures;
extern crate http;

use aitch::servers::hyper::Server;
use aitch::{middlewares, Responder, ResponseBuilder, Result};
use futures::IntoFuture;
use http::Request;

fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
    let vec = "Hello from the future!".to_owned();
    resp.body(vec).into_future()
}

fn main() -> Result<()> {
    let wrapped = middlewares::with_stdout_logging(handler);

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped)?.run()
}
