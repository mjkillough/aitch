extern crate aitch;
extern crate futures;
extern crate http;

use aitch::ResponseBuilder;
use futures::Future;
use http::Request;

fn handler(
    _req: &mut Request<Vec<u8>>,
    mut resp: ResponseBuilder,
) -> impl Future<Item = http::Response<Vec<u8>>, Error = http::Error> {
    let v = "Hello from the future!".as_bytes().to_owned();
    let r = resp.body(v.into()).unwrap();
    futures::future::ok(r)
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);
    aitch::Server::new(addr, handler).run();
}
