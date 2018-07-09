extern crate aitch;
extern crate http;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use aitch::servers::hyper::Server;
use aitch::{middlewares, Json, Responder, ResponseBuilder, Result};
use http::Request;

#[derive(Serialize, Deserialize)]
struct Message {
    message: String,
    from: String,
}

fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
    let msg = Message {
        message: "Hello, world!".to_owned(),
        from: "aitch's JSON handling".to_owned(),
    };
    resp.header(http::header::CONTENT_TYPE, "application/json")
        .body(Json(msg))
}

fn main() -> Result<()> {
    let wrapped = middlewares::with_logging(handler);

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped).run()
}
