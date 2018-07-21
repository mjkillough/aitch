use std::sync::Arc;

use futures::Future;
use http;

use {Body, BodyStream, BoxedResponse, Error, Responder, ResponseBuilder};

/// A trait indicating that the type can be used to handler to a HTTP request.
///
/// The `Handler` trait is most commonly used through functions of the following type, for which it
/// is automatically implemented:
///
/// ```ignore
/// Fn(http::Request<impl Body>, http::response::Builder) -> impl Responder
/// ```
///
/// It can also be implemented manually for more complex types, which store state associated with
/// the handler.
///
/// # Example
///
/// A function which implements the `Handler` trait:
///
/// ```
/// # extern crate aitch;
/// # extern crate http;
/// #
/// # use aitch::{Responder, ResponseBuilder};
/// # use http::Request;
/// #
/// fn handler(_: http::Request<()>, mut resp: ResponseBuilder) -> impl Responder {
///     resp.body("Hello, world".to_owned())
/// }
/// ```
///
/// A more complex type can also implement the `Handler` trait directly:
///
/// ```
/// # extern crate aitch;
/// # extern crate http;
/// #
/// # use aitch::{Handler, Responder, ResponseBuilder};
/// # use http::Request;
/// #
/// struct ComplexHandler {
///     message: String
/// }
///
/// impl Handler<()> for ComplexHandler {
///     type Resp = http::Result<http::Response<String>>;
///
///     fn handle(&self, _: Request<()>, mut resp: ResponseBuilder) -> Self::Resp {
///         resp.body(self.message.clone())
///     }
/// }
/// ```
///
/// # Storing a `Handler`
///
/// If you need to store a `Handler`, the [`BoxedHandler`] type (and corresponding [`box_handler`]
/// function) can be used to erase the generic types of the handler.
///
/// This can be particularly useful if you need to store multiple handlers (such as in a request
/// router) which may deal with different request/response bodies, or use different [`Responder`]
/// types.
///
/// [`BoxedHandler`]: type.BoxedHandler.html
/// [`box_handler`]: fn.box_handler.html
/// [`Responder`]: trait.Responder.html
pub trait Handler<ReqBody>
where
    ReqBody: Body,
    Self: Send + Sync + 'static,
{
    /// The `Responder` type returned by this `Handler`.
    type Resp: Responder;

    /// Handles an incoming HTTP request, returning a `Responder` describing a HTTP response.
    fn handle(&self, http::Request<ReqBody>, ResponseBuilder) -> Self::Resp;
}

impl<Func, ReqBody, Resp> Handler<ReqBody> for Func
where
    Func: Fn(http::Request<ReqBody>, ResponseBuilder) -> Resp + Send + Sync + 'static,
    ReqBody: Body,
    Resp: Responder,
{
    type Resp = Resp;

    fn handle(&self, req: http::Request<ReqBody>, resp: ResponseBuilder) -> Resp {
        (self)(req, resp)
    }
}

/// A `Box<Handler>` with all types erased.
///
/// This allows multiple handlers with different type signature (e.g. different body/responder
/// types) to be stored in the same data-structure.
///
/// # Example
///
/// ```
/// # extern crate aitch;
/// # extern crate http;
/// #
/// # use std::collections::HashMap;
/// #
/// # use aitch::{
/// #     BodyStream, BoxedHandler, BoxedResponse, Handler, Responder, ResponseBuilder
/// # };
/// # use http::Request;
/// #
/// /// A Very Simple Router which matches handlers based on the full path in request URIs.
/// struct VerySimpleRouter(pub HashMap<String, BoxedHandler>);
///
/// impl Handler<BodyStream> for VerySimpleRouter {
///     type Resp = BoxedResponse;
///
///     fn handle(&self, req: Request<BodyStream>, mut resp: ResponseBuilder) -> BoxedResponse {
///         match self.0.get(req.uri().path()) {
///             Some(handler) => handler.handle(req, resp),
///             // Note the use of `.into_response()`, which boxes the response to erase any
///             // type variables used by the responder, matching the response type used by
///             // `BoxedHandler` above.
///             None => resp.status(http::StatusCode::NOT_FOUND).body(()).into_response(),
///         }
///     }
/// }
/// ```
///
/// See the [`SimpleRouter` source code] for a (slightly) more complete example.
///
/// [`SimpleRouter` source code]: ../src/aitch/middlewares/router.rs.html
pub type BoxedHandler = Box<Handler<BodyStream, Resp = BoxedResponse>>;

fn map_request_body<B1, B2>(
    req: http::Request<B1>,
) -> impl Future<Item = http::Request<B2>, Error = Error>
where
    B1: Body,
    B2: Body,
{
    let (parts, body) = req.into_parts();
    body.into_body::<B2>()
        .map(move |body| http::Request::from_parts(parts, body))
}

/// Creates a [`BoxedHandler`] from any [`Handler`].
///
/// See the [`BoxedHandler`] for more details about how this can be used.
///
/// [`BoxedHandler`]: type.BoxedHandler.html
/// [`Handler`]: trait.Handler.html
pub fn box_handler<B: Body>(handler: impl Handler<B>) -> BoxedHandler {
    let handler = Arc::new(handler);
    let closure = move |req, resp| -> BoxedResponse {
        let handler = handler.clone();
        let resp =
            map_request_body(req).and_then(move |req| handler.handle(req, resp).into_response());
        Box::new(resp)
    };
    Box::new(closure)
}
