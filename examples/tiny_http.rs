extern crate aitch;
extern crate http;

use aitch::servers::tiny_http::Server;
use aitch::{middlewares, Responder, ResponseBuilder, Result};
use http::Request;

fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
    resp.body("Hello, world!".to_owned())
}

fn main() -> Result<()> {
    let wrapped = middlewares::with_stdout_logging(handler);

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped)?.run()
}
