use http;

use super::{FromHttpResponse, HttpBody, IntoResponse, ResponseBuilder};
use async::{AsyncBody, FutureResponse};
use sync::SyncBody;

pub trait Handler<Body, Resp>
where
    Resp: FromHttpResponse<Body>,
    Body: HttpBody,
{
    fn handle(&self, &mut http::Request<Body>, ResponseBuilder) -> Resp;
}


pub trait AsyncHandler: Handler<AsyncBody, FutureResponse<AsyncBody>> {}

impl<H> AsyncHandler for H
where
    H: Handler<AsyncBody, FutureResponse<AsyncBody>>,
{
}


pub trait SyncHandler: Handler<SyncBody, http::Response<SyncBody>> {}

impl<H> SyncHandler for H
where
    H: Handler<SyncBody, http::Response<SyncBody>>,
{
}


// We have separate SyncHandlerFunc/AsyncHandlerFunc types because:
//  - `HandlerFunc<Func, Body, Resp>` requires a `PhantomData` to use the type
//    parameters, so stops us from using a tuple struct.
//  - It stops us from hitting rust/issues#41078 when passing a closure. If
//    `Func` is generic over `Body`, then the compiler isn't smart enough to
//    infer the right lifetime. We'd need all closures to specify the type of
//    `req` to work around this.


pub struct SyncHandlerFunc<Func, Resp>(pub Func)
where
    Func: Fn(&mut http::Request<SyncBody>, ResponseBuilder) -> Resp;

impl<Func, Resp, HandlerResp> Handler<SyncBody, HandlerResp> for SyncHandlerFunc<Func, Resp>
where
    Func: Fn(&mut http::Request<SyncBody>, ResponseBuilder) -> Resp,
    Resp: IntoResponse<HandlerResp>,
    HandlerResp: FromHttpResponse<SyncBody>,
{
    fn handle(&self, req: &mut http::Request<SyncBody>, resp: ResponseBuilder) -> HandlerResp {
        (self.0)(req, resp).into_response()
    }
}


pub struct AsyncHandlerFunc<Func, Resp>(pub Func)
where
    Func: Fn(&mut http::Request<AsyncBody>, ResponseBuilder) -> Resp;

impl<Func, Resp, HandlerResp> Handler<AsyncBody, HandlerResp> for AsyncHandlerFunc<Func, Resp>
where
    Func: Fn(&mut http::Request<AsyncBody>, ResponseBuilder) -> Resp,
    Resp: IntoResponse<HandlerResp>,
    HandlerResp: FromHttpResponse<AsyncBody>,
{
    fn handle(&self, req: &mut http::Request<AsyncBody>, resp: ResponseBuilder) -> HandlerResp {
        (self.0)(req, resp).into_response()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use futures::{self, Future, Stream};

    fn sync_handler_returning_response(
        _req: &mut http::Request<SyncBody>,
        mut resp: ResponseBuilder,
    ) -> http::Response<SyncBody> {
        resp.body(SyncBody::empty()).unwrap()
    }

    fn sync_handler_returning_result(
        _req: &mut http::Request<SyncBody>,
        mut resp: ResponseBuilder,
    ) -> http::Response<SyncBody> {
        resp.body(SyncBody::empty()).unwrap()
    }

    fn async_handler(
        _req: &mut http::Request<AsyncBody>,
        mut resp: ResponseBuilder,
    ) -> FutureResponse<AsyncBody> {
        let resp = resp.body(AsyncBody::empty()).unwrap();
        Box::new(futures::future::ok(resp))
    }

    fn request<Body>() -> http::Request<Body>
    where
        Body: HttpBody,
    {
        http::Request::builder().body(Body::empty()).unwrap()
    }

    #[test]
    fn test_sync_handler_func_returning_response() {
        let handler = SyncHandlerFunc(sync_handler_returning_response);
        let resp = handler.handle(&mut request(), ResponseBuilder::new());
        assert_eq!(resp.body(), &SyncBody::empty());
    }

    #[test]
    fn test_sync_handler_func_returning_result() {
        let handler = SyncHandlerFunc(sync_handler_returning_result);
        let resp = handler.handle(&mut request(), ResponseBuilder::new());
        assert_eq!(resp.body(), &SyncBody::empty());
    }

    #[test]
    fn test_sync_handler_closure() {
        let handler = SyncHandlerFunc(|req, resp| sync_handler_returning_response(req, resp));
        let resp = handler.handle(&mut request(), ResponseBuilder::new());
        assert_eq!(resp.body(), &SyncBody::empty());
    }

    #[test]
    fn test_async_handler() {
        let handler = AsyncHandlerFunc(async_handler);
        let fut = handler.handle(&mut request(), ResponseBuilder::new());
        let resp = fut.wait().unwrap();
        let (_, body_stream) = resp.into_parts();
        let body = body_stream.concat2().wait().unwrap().to_vec();
        assert_eq!(body, vec![],);
    }
}
