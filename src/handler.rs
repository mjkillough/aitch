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

pub type BoxedHandler = Box<Handler<BodyStream, Resp = BoxedResponse>>;

fn map_request_body<B1, B2>(
    req: http::Request<B1>,
) -> impl Future<Item = http::Request<B2>, Error = Error>
where
    B1: Body,
    B2: Body,
{
    let (parts, body) = req.into_parts();
    B2::from_stream(body.into_stream()).map(move |body| http::Request::from_parts(parts, body))
}

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
