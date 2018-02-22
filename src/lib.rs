#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate http;
extern crate hyper;

mod errors;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures::{Future, Stream};
use hyper::server::{Request as HyperRequest, Response as HyperResponse};

use errors::*;


pub type Request = http::Request<Vec<u8>>;
pub type Response = http::Response<Vec<u8>>;
pub type ResponseBuilder = http::response::Builder;


pub struct SimpleRouter {
    handlers: HashMap<String, Box<Handler>>,
}

impl SimpleRouter {
    pub fn new() -> SimpleRouter {
        SimpleRouter {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<S, H>(&mut self, pattern: S, handler: H)
    where
        S: Into<String>,
        H: Handler + 'static,
    {
        let pattern = pattern.into();
        if self.handlers.contains_key(&pattern) {
            panic!("SimpleRouter: Tried to register pattern twice: {}", pattern);
        }
        self.handlers.insert(pattern, Box::new(handler));
    }

    pub fn to_handler(self) -> Arc<Fn(&mut Request, ResponseBuilder) -> Response> {
        Arc::new(move |req, resp| self.handle(req, resp))
    }
}

impl Handler for SimpleRouter {
    fn handle(&self, req: &mut Request, mut resp: ResponseBuilder) -> Response {
        let matching = self.handlers
            .iter()
            .filter(|&(pattern, _)| req.uri().path().starts_with(pattern))
            .max_by(|&(pattern1, _), &(pattern2, _)| pattern1.cmp(pattern2));

        match matching {
            Some((_, handler)) => handler.handle(req, resp),
            None => resp.status(http::StatusCode::NOT_FOUND)
                .body(Vec::new())
                .into_response(),
        }
    }
}


pub trait Responder {
    fn into_response(self) -> Response;
}

impl Responder for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl Responder for http::Result<Response> {
    fn into_response(self) -> Response {
        self.unwrap_or_else(|_| {
            http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Vec::new())
                .unwrap()
        })
    }
}


pub trait Handler {
    fn handle(&self, &mut Request, ResponseBuilder) -> Response;
}


pub struct HandlerFunc<Func, Resp>(pub Func)
where
    Func: Fn(&mut Request, ResponseBuilder) -> Resp,
    Resp: Responder;

impl<Func, Resp> Handler for HandlerFunc<Func, Resp>
where
    Func: Fn(&mut Request, ResponseBuilder) -> Resp,
    Resp: Responder,
{
    fn handle(&self, req: &mut Request, resp: ResponseBuilder) -> Response {
        (self.0)(req, resp).into_response()
    }
}


pub struct Server<H>
where
    H: Handler + 'static,
{
    addr: SocketAddr,
    handler: Arc<H>,
}

impl<H> Server<H>
where
    H: Handler + 'static,
{
    pub fn new(addr: SocketAddr, handler: H) -> Server<H> {
        let handler = Arc::new(handler);
        Server { addr, handler }
    }

    pub fn run(self) -> Result<()> {
        let addr = self.addr;
        hyper::server::Http::new()
            .bind(&addr, move || {
                Ok(Service {
                    handler: self.handler.clone(),
                })
            })?
            .run()?;
        Ok(())
    }
}

struct Service<H>
where
    H: Handler + 'static,
{
    handler: Arc<H>,
}

impl<H> hyper::server::Service for Service<H>
where
    H: Handler + 'static,
{
    type Request = HyperRequest;
    type Response = HyperResponse;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: HyperRequest) -> Self::Future {
        let handler = self.handler.clone();
        let req: http::Request<hyper::Body> = req.into();
        let (parts, body) = req.into_parts();
        let fut = body.concat2().and_then(move |body| {
            let req = Request::from_parts(parts, body.to_vec());
            let builder = http::Response::builder();
            let response = handler.handle(&mut req.into(), builder);
            let hyper_response = response.map(|body| hyper::Body::from(hyper::Chunk::from(body)));
            futures::future::ok(hyper_response.into())
        });
        Box::new(fut)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use http;

    fn handler1(_req: &mut Request, mut resp: ResponseBuilder) -> http::Result<Response> {
        resp.body("1".as_bytes().to_owned())
    }

    fn handler2(_req: &mut Request, mut resp: ResponseBuilder) -> Response {
        resp.body("2".as_bytes().to_owned()).unwrap()
    }

    struct Handler3;

    impl Handler for Handler3 {
        fn handle(&self, _req: &mut Request, mut resp: ResponseBuilder) -> Response {
            resp.body("3".as_bytes().to_owned()).unwrap()
        }
    }

    #[test]
    fn test_function_can_return_responder() {
        Server::new("127.0.0.1:8000".parse().unwrap(), HandlerFunc(handler1));
        Server::new("127.0.0.1:8000".parse().unwrap(), HandlerFunc(&handler1));
    }

    #[test]
    fn test_function_can_return_response() {
        Server::new("127.0.0.1:8000".parse().unwrap(), HandlerFunc(handler2));
        Server::new("127.0.0.1:8000".parse().unwrap(), HandlerFunc(&handler2));
    }

    #[test]
    fn test_closure() {
        Server::new(
            "127.0.0.1:8000".parse().unwrap(),
            HandlerFunc(|req, resp| handler2(req, resp)),
        );
    }

    #[test]
    fn test_handler() {
        Server::new("127.0.0.1:8000".parse().unwrap(), Handler3 {});
    }

    #[test]
    fn test_router() {
        let mut router = SimpleRouter::new();
        router.register_handler("/handler1", HandlerFunc(handler1));
        router.register_handler("/handler2a", HandlerFunc(handler2));
        router.register_handler("/handler2b", HandlerFunc(|req, resp| handler2(req, resp)));
        router.register_handler("/handler3", Handler3 {});

        let route = |path| {
            router
                .handle(
                    &mut http::Request::get(path).body(vec![]).unwrap(),
                    ResponseBuilder::new(),
                )
                .body()
                .clone()
        };

        assert_eq!(route("/handler1"), "1".as_bytes());
        assert_eq!(route("/handler2a"), "2".as_bytes());
        assert_eq!(route("/handler2b"), "2".as_bytes());
        assert_eq!(route("/handler3"), "3".as_bytes());
    }
}
