extern crate aitch;
extern crate http;

use aitch::servers::hyper::Server;
use aitch::{middlewares, Responder, ResponseBuilder, Result};
use http::Request;

fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
    resp.body("Hello, world!".to_owned())
}

fn main() -> Result<()> {
    let wrapped = handler;

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped).run()
}
