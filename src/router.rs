use std::collections::HashMap;

use futures::IntoFuture;
use http;

use {
    box_handler, response_with_status, Body, BodyStream, BoxedHandler, BoxedResponse, Handler,
    Responder, ResponseBuilder,
};

pub struct SimpleRouter {
    handlers: HashMap<String, BoxedHandler>,
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
        self.handlers.insert(pattern, box_handler(handler));
    }

    pub fn handler(&self, uri: &http::Uri) -> Option<(&String, &BoxedHandler)> {
        self.handlers
            .iter()
            .filter(|&(pattern, _)| uri.path().starts_with(pattern))
            .max_by(|&(pattern1, _), &(pattern2, _)| pattern1.cmp(pattern2))
    }
}

impl Handler<BodyStream> for SimpleRouter {
    type Resp = BoxedResponse;

    fn handle(&self, req: http::Request<BodyStream>, resp: ResponseBuilder) -> BoxedResponse {
        match self.handler(req.uri()) {
            Some((_, handler)) => handler.handle(req, resp),
            None => response_with_status(http::StatusCode::NOT_FOUND).into_response(),
        }
    }
}
