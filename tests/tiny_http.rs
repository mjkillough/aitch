extern crate aitch;
extern crate http;
extern crate reqwest;

use std::net::SocketAddr;
use std::thread;

use aitch::{Body, Handler, ResponseBuilder};
use http::Request;

struct Server {
    addr: SocketAddr,
}

impl Server {
    fn start_in_thread<B: Body>(handler: impl Handler<B>) -> Self {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = aitch::servers::tiny_http::Server::new(addr, handler).unwrap();
        let addr = server.addr();
        thread::spawn(move || server.run());
        Server { addr }
    }

    fn path(&self, path: &str) -> String {
        format!("http://{}/{}", self.addr, path)
    }
}

#[test]
fn echo_server() {
    let server = Server::start_in_thread(|req: Request<String>, mut resp: ResponseBuilder| {
        resp.body(req.into_body())
    });

    let client = reqwest::Client::new();
    let mut resp = client
        .post(&server.path("/"))
        .body("some body")
        .send()
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);

    let body = resp.text().unwrap();
    assert_eq!(body, "some body");
}
