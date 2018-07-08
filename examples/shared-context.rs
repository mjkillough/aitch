extern crate aitch;
extern crate http;

use std::sync::Arc;

use aitch::{middlewares, Responder, ResponseBuilder};
use http::Request;

struct Context {
    message: String,
}

fn handler(ctx: Arc<Context>, _req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
    resp.body(ctx.message.clone())
}

fn main() {
    let ctx = Arc::new(Context {
        message: "Hello from a world with context!".to_owned(),
    });
    let handler = middlewares::with_context(ctx, handler);
    let wrapped = middlewares::logging_handler(handler);

    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::servers::HyperServer::new(addr, wrapped).run();
}
