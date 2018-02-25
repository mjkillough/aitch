use http;

use super::{AsyncBody, EmptyBody, FromHttpResponse, FutureResponse, IntoResponse, ResponseBuilder,
            SyncBody};


pub trait Handler<Body, Resp>
where
    Resp: FromHttpResponse<Body>,
    Body: EmptyBody,
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
