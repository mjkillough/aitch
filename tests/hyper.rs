extern crate aitch;
extern crate http;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::net::SocketAddr;
use std::thread;

use aitch::{handlers, middlewares, Body, Handler, Json, ResponseBuilder};
use http::Request;

struct Server {
    addr: SocketAddr,
}

impl Server {
    fn start_in_thread<B: Body>(handler: impl Handler<B>) -> Self {
        let addr = "127.0.0.1:0".parse().unwrap();
        let server = aitch::servers::hyper::Server::new(addr, handler).unwrap();
        let addr = server.addr();
        thread::spawn(move || server.run());
        Server { addr }
    }

    fn path(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
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

#[test]
fn json() {
    #[derive(Deserialize, Serialize)]
    struct ReqBody {
        message: String,
    }

    let server =
        Server::start_in_thread(|req: Request<Json<ReqBody>>, mut resp: ResponseBuilder| {
            let body = req.into_body().json();
            resp.body(body.message)
        });

    let client = reqwest::Client::new();
    let mut resp = client
        .post(&server.path("/"))
        .body("{\"message\": \"some message\"}")
        .send()
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);

    let body = resp.text().unwrap();
    assert_eq!(body, "some message");
}

#[test]
fn status_code() {
    let server = Server::start_in_thread(|_: Request<()>, mut resp: ResponseBuilder| {
        resp.status(http::StatusCode::NOT_FOUND).body(())
    });

    let resp = reqwest::get(&server.path("/")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NotFound);
}

#[test]
fn router() {
    let handler =
        |t: &'static str| move |_: Request<()>, mut resp: ResponseBuilder| resp.body(t.to_owned());
    let mut router = middlewares::SimpleRouter::new();
    router.register_handler("/handler1", handler("1"));
    router.register_handler("/handler2", handler("2"));
    router.register_handler("/handler11", handler("11"));

    let server = Server::start_in_thread(router);

    let resp = reqwest::get(&server.path("/")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NotFound);

    let mut resp = reqwest::get(&server.path("/handler1")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);
    let body = resp.text().unwrap();
    assert_eq!(body, "1");

    let mut resp = reqwest::get(&server.path("/handler11")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);
    let body = resp.text().unwrap();
    assert_eq!(body, "11");

    let mut resp = reqwest::get(&server.path("/handler2")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);
    let body = resp.text().unwrap();
    assert_eq!(body, "2");
}

#[test]
fn static_files_handler() {
    let handler = handlers::static_files::static_files_handler("./examples/static-files/").unwrap();
    let server = Server::start_in_thread(handler);

    let mut resp = reqwest::get(&server.path("/hello.txt")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);
    let body = resp.text().unwrap();
    assert_eq!(body, "hello");

    let resp = reqwest::get(&server.path("/not-found")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NotFound);
}

#[test]
fn static_file_handler() {
    let handler = handlers::static_files::static_file_handler("./examples/static-files/hello.txt");
    let server = Server::start_in_thread(handler);

    let mut resp = reqwest::get(&server.path("/hello.txt")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);
    let body = resp.text().unwrap();
    assert_eq!(body, "hello");

    // Path doesn't matter.
    let mut resp = reqwest::get(&server.path("/not-file")).unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::Ok);
    let body = resp.text().unwrap();
    assert_eq!(body, "hello");
}
