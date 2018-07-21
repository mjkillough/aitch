//! A collection of useful HTTP middlewares.

mod router;

use futures::Future;
use http;

use {Body, Error, Handler, Responder, ResponseBuilder};

pub use self::router::SimpleRouter;

/// Middleware which outputs details of HTTP requests/responses to stdout.
///
/// This middleware wraps another HTTP handler, and logs details about each HTTP request (method and
/// URI) and the associated response (status) to stdout. The logging only occurs when the
/// [`Responder`] returned from the [`Handler`] resolved to a `http::Response`.
///
/// This middleware is intended to help during development. It is expected that any production
/// application creates its own version of this middleware, to integrate it into the application's
/// logging infrastructure.
///
/// [`Responder`]: ../trait.Responder.html [`Handler`]: ../trait.Handler.html [`http::Response`]:
/// https://docs.rs/http/0.1.7/http/response/struct.Response.html
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
/// fn handler(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
///     resp.body("Hello, world!".to_owned())
/// }
///
/// fn main() -> Result<()> {
///     let wrapped = middlewares::with_stdout_logging(handler);
///
///     let addr = "127.0.0.1:3000".parse()?;
///     println!("Listening on http://{}", addr);
///     Server::new(addr, wrapped)?.run()
/// }
/// ```
///
/// Which outputs the following in response to requests:
///
/// ```ignore
/// Listening on http://127.0.0.1:3000"
/// GET / 200
/// GET /path/doesnt/matter/in/this/example 200
/// ...
/// ```
pub fn with_stdout_logging<B: Body>(handler: impl Handler<B>) -> impl Handler<B> {
    move |req: http::Request<B>, resp: ResponseBuilder| {
        let method = req.method().clone();
        let uri = req.uri().clone();

        handler.handle(req, resp).into_response().map(move |resp| {
            println!("{} {} {}", method, uri.path(), resp.status());
            resp
        })
    }
}

// TODO: Determine whether this pattern is useful, and consider exporting.
pub(crate) fn with_error_handling<B, F, R>(handler: impl Handler<B>, func: F) -> impl Handler<B>
where
    B: Body,
    F: Fn(Error) -> R + Clone + Send + Sync + 'static,
    R: Responder,
{
    with_context(func, move |func, req, resp| {
        handler
            .handle(req, resp)
            .into_response()
            .or_else(move |err| func(err).into_response())
    })
}

/// A middleware which injects shared context into HTTP handlers.
///
/// The `with_context` function is a convenience that makes writing handlers with shared state as
/// easy as possible. In many HTTP applications, handlers need access to shared context/state, such
/// as database connection pools, or configuration information.
///
/// When using `with_context`, handler functions are written using the slightly different type
/// signature, which accepts a context type as the first argument:
///
/// ```ignore
/// Fn(ContextType, http::Request<impl Body>, ResponseBuilder) -> impl Responder
/// ```
///
/// They are then wrapped with `with_context(ctx, func)`, which returns a [`Handler`] which can be
/// passed to middlewares/servers.
///
/// The only constraints on the context type, are that it implements `Clone + Send + Sync`. In many
/// applications, this type will be an [`Arc`], or something that uses [`Arc`] internally.
///
/// [`Handler`]: ../trait.Handler.html
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
///
/// # Example
///
/// ```no_run
/// extern crate aitch;
/// extern crate http;
///
/// use std::sync::Arc;
///
/// use aitch::servers::hyper::Server;
/// use aitch::{middlewares, Responder, ResponseBuilder, Result};
/// use http::Request;
///
/// struct Context {
///     message: String,
///     // In many applications, this context would also contain a database
///     // connection pool, configuration data, and anything other state that
///     // needs to be shared between handlers:
///     // pool: DatabasePool,
///     // config: Config,
/// }
///
/// fn handler(ctx: Arc<Context>, _req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
///     resp.body(ctx.message.clone())
/// }
///
/// fn main() -> Result<()> {
///     let ctx = Arc::new(Context {
///         message: "Hello from a world with context!".to_owned(),
///     });
///     let handler = middlewares::with_context(ctx, handler);
///     let wrapped = middlewares::with_stdout_logging(handler);
///
///     let addr = "127.0.0.1:3000".parse()?;
///     println!("Listening on http://{}", addr);
///     Server::new(addr, wrapped)?.run()
/// }
/// ```

pub fn with_context<Ctx, Func, ReqBody, Resp>(ctx: Ctx, handler: Func) -> impl Handler<ReqBody>
where
    Ctx: Clone + Send + Sync + 'static,
    Func: Fn(Ctx, http::Request<ReqBody>, ResponseBuilder) -> Resp + Send + Sync + 'static,
    ReqBody: Body,
    Resp: Responder,
{
    move |req, resp| handler(ctx.clone(), req, resp)
}
