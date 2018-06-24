use std::collections::HashMap;

use futures::IntoFuture;
use http;

use traits::HttpBody;
use {box_response, BoxedResponse, Handler, ResponseBuilder};

pub struct SimpleRouter<Body>
where
    Body: HttpBody,
{
    handlers: HashMap<String, Box<Handler<Body, BoxedResponse<Body>>>>,
}

impl<Body> SimpleRouter<Body>
where
    Body: HttpBody + 'static,
{
    pub fn new() -> SimpleRouter<Body> {
        SimpleRouter {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<S, H, Resp>(&mut self, pattern: S, handler: H)
    where
        S: Into<String>,
        H: Handler<Body, Resp> + 'static,
        Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error> + 'static,
    {
        let pattern = pattern.into();
        if self.handlers.contains_key(&pattern) {
            panic!("SimpleRouter: Tried to register pattern twice: {}", pattern);
        }
        self.handlers
            .insert(pattern, Box::new(box_response(handler)));
    }

    pub fn handler(
        &self,
        req: &http::Request<Body>,
    ) -> Option<(&String, &Box<Handler<Body, BoxedResponse<Body>>>)> {
        self.handlers
            .iter()
            .filter(|&(pattern, _)| req.uri().path().starts_with(pattern))
            .max_by(|&(pattern1, _), &(pattern2, _)| pattern1.cmp(pattern2))
    }
}

impl<Body> Handler<Body, BoxedResponse<Body>> for SimpleRouter<Body>
where
    Body: HttpBody + 'static,
{
    fn handle(
        &self,
        req: &mut http::Request<Body>,
        mut resp: ResponseBuilder,
    ) -> BoxedResponse<Body> {
        match self.handler(req) {
            Some((_, handler)) => handler.handle(req, resp),
            None => Box::new(
                resp.status(http::StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .into_future(),
            ),
        }
    }
}
