use std::collections::HashMap;

use futures::IntoFuture;
use http;

use {genericify, Body, BodyStream, Handler};

pub struct SimpleRouter {
    handlers: HashMap<String, Box<Handler<BodyStream>>>,
}

impl SimpleRouter {
    pub fn new() -> SimpleRouter {
        SimpleRouter {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<S, H, ReqBody>(&mut self, pattern: S, handler: H)
    where
        S: Into<String>,
        H: Handler<ReqBody>,
        ReqBody: Body,
    {
        let pattern = pattern.into();
        if self.handlers.contains_key(&pattern) {
            panic!("SimpleRouter: Tried to register pattern twice: {}", pattern);
        }
        self.handlers.insert(pattern, Box::new(genericify(handler)));
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

impl<ReqBody> Handler<ReqBody> for SimpleRouter
where
    ReqBody: ReqBody,
{
    fn handle(&self, req: http::Request<ReqBody>, mut resp: ResponseBuilder) -> impl Responder {
        match self.handler(&req) {
            Some((_, handler)) => handler.handle(req, resp),
            None => Box::new(
                resp.status(http::StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .into_future(),
            ),
        }
    }
}
