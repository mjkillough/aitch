use std::sync::Arc;

use futures::Future;
use http;

use {Body, BodyStream, BoxedResponse, Error, Responder, ResponseBuilder};

pub trait Handler<ReqBody>
where
    ReqBody: Body,
    Self: Send + Sync + 'static,
{
    type Resp: Responder;

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
