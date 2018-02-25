use std::collections::HashMap;

use http;

use super::{Handler, ResponseBuilder};
use traits::{FromHttpResponse, HttpBody};


pub struct SimpleRouter<Body, Resp>
where
    Resp: FromHttpResponse<Body>,
    Body: HttpBody,
{
    handlers: HashMap<String, Box<Handler<Body, Resp>>>,
}

impl<Body, Resp> SimpleRouter<Body, Resp>
where
    Resp: FromHttpResponse<Body>,
    Body: HttpBody,
{
    pub fn new() -> SimpleRouter<Body, Resp> {
        SimpleRouter {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<S, H>(&mut self, pattern: S, handler: H)
    where
        S: Into<String>,
        H: Handler<Body, Resp> + 'static,
    {
        let pattern = pattern.into();
        if self.handlers.contains_key(&pattern) {
            panic!("SimpleRouter: Tried to register pattern twice: {}", pattern);
        }
        self.handlers.insert(pattern, Box::new(handler));
    }
}

impl<Body, Resp> Handler<Body, Resp> for SimpleRouter<Body, Resp>
where
    Resp: FromHttpResponse<Body>,
    Body: HttpBody,
{
    fn handle(&self, req: &mut http::Request<Body>, mut resp: ResponseBuilder) -> Resp {
        let matching = self.handlers
            .iter()
            .filter(|&(pattern, _)| req.uri().path().starts_with(pattern))
            .max_by(|&(pattern1, _), &(pattern2, _)| pattern1.cmp(pattern2));

        match matching {
            Some((_, handler)) => handler.handle(req, resp),
            None => Resp::from_http_response(
                resp.status(http::StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap(),
            ),
        }
    }
}
