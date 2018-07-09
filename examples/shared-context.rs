extern crate aitch;
extern crate http;

use std::sync::Arc;

use aitch::servers::hyper::Server;
use aitch::{middlewares, Responder, ResponseBuilder, Result};
use http::Request;

struct Context {
    message: String,
}

fn handler(ctx: Arc<Context>, _req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
    resp.body(ctx.message.clone())
}

fn main() -> Result<()> {
    let ctx = Arc::new(Context {
        message: "Hello from a world with context!".to_owned(),
    });
    let handler = middlewares::with_context(ctx, handler);
    let wrapped = middlewares::with_logging(handler);

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped).run()
}
