use std::collections::HashMap;

use http;

use {
    box_handler, Body, BodyStream, BoxedHandler, BoxedResponse, Handler, Responder, ResponseBuilder,
};

/// A simple request router, which determines which handler to call based on the request URI's path.
///
/// This is a simple request router, inspired by Go's [`net/http` `ServeMux`].
///
/// It matches the path component of a HTTP request's URI against a collection of registered path
/// prefixes, and calls the handler whose path prefix most closely matches. (e.g. if both `/path/a`
/// and `/path/ab` are registered, a request for `/path/abc/` will call the latter).
///
/// This router is intended primarily to serve as an example of writing complex middleware using
/// aitch, and library users are encouraged to read its [source code].
///
/// The router is not intended for production applications, and lacks many features that are present
/// in the routers of other web frameworks/toolkits.
///
/// [`net/http` `ServeMux`]: https://golang.org/pkg/net/http/#ServeMux
/// [source code]: ../../src/aitch/middlewares/router.rs.html
///
/// # Example
///
/// ```no_run
/// extern crate aitch;
/// extern crate http;
///
/// use aitch::servers::hyper::Server;
/// use aitch::{middlewares, Responder, ResponseBuilder, Result};
/// use http::Request;
///
/// fn handler1(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
///     resp.body("Handler 1!".to_owned())
/// }
///
/// fn handler2(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
///     resp.body("Handler 2!".to_owned())
/// }
///
/// fn main() -> Result<()> {
///     let mut router = middlewares::SimpleRouter::new();
///     router.register_handler("/", handler1);
///     router.register_handler("/handler2", handler2);
///
///     let handler = middlewares::with_stdout_logging(router);
///
///     let addr = "127.0.0.1:3000".parse()?;
///     println!("Listening on http://{}", addr);
///     Server::new(addr, handler)?.run()
/// }
/// ```

#[derive(Default)]
pub struct SimpleRouter {
    handlers: HashMap<String, BoxedHandler>,
}

impl SimpleRouter {
    /// Creates a new `SimpleRouter`, with no routes registered.
    pub fn new() -> Self {
        SimpleRouter::default()
    }

    /// Registers a handler with the given pattern.
    ///
    /// This method registers a new handler with the router, using the provided pattern.
    ///
    /// See the [module level documentation] for more details on how patterns are matched.
    ///
    /// [module level documentation]: ./index.html
    ///
    /// # Panics
    ///
    /// This method panics if a handler is already registered with the provided pattern.
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

    /// Returns the handler to be used for a request with the given URI.
    ///
    /// Returns `None` if no handler matches the URI.
    pub fn handler(&self, uri: &http::Uri) -> Option<(&String, &BoxedHandler)> {
        self.handlers
            .iter()
            .filter(|&(pattern, _)| uri.path().starts_with(pattern))
            .max_by_key(|&(pattern, _)| pattern.len())
    }
}

impl Handler<BodyStream> for SimpleRouter {
    type Resp = BoxedResponse;

    fn handle(&self, req: http::Request<BodyStream>, mut resp: ResponseBuilder) -> BoxedResponse {
        match self.handler(req.uri()) {
            Some((_, handler)) => handler.handle(req, resp),
            None => resp.status(http::StatusCode::NOT_FOUND)
                .body(())
                .into_response(),
        }
    }
}
