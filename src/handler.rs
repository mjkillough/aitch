use std::marker::PhantomData;

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


pub struct HandlerFunc<Func, Body, Resp>
where
    Func: Fn(&mut http::Request<Body>, ResponseBuilder) -> Resp,
    Body: EmptyBody,
{
    func: Func,
    marker: PhantomData<(Body, Resp)>,
}

impl<Func, Body, Resp> From<Func> for HandlerFunc<Func, Body, Resp>
where
    Func: Fn(&mut http::Request<Body>, ResponseBuilder) -> Resp,
    Body: EmptyBody,
{
    fn from(func: Func) -> HandlerFunc<Func, Body, Resp> {
        let marker = PhantomData;
        HandlerFunc { func, marker }
    }
}


impl<Func, Body, Resp, HandlerResp> Handler<Body, HandlerResp> for HandlerFunc<Func, Body, Resp>
where
    Func: Fn(&mut http::Request<Body>, ResponseBuilder) -> Resp,
    Resp: IntoResponse<HandlerResp>,
    Body: EmptyBody,
    HandlerResp: FromHttpResponse<Body>,
{
    fn handle(&self, req: &mut http::Request<Body>, resp: ResponseBuilder) -> HandlerResp {
        (self.func)(req, resp).into_response()
    }
}
