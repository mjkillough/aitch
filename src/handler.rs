use http;

use {Body, Responder, ResponseBuilder};

pub trait Handler<ReqBody, Resp, RespBody>
where
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
    Self: Send + Sync + 'static,
{
    fn handle(&self, http::Request<ReqBody>, ResponseBuilder) -> Resp;
}

impl<Func, ReqBody, Resp, RespBody> Handler<ReqBody, Resp, RespBody> for Func
where
    ReqBody: Body,
    Resp: Responder<RespBody>,
    RespBody: Body,
    Func: Fn(http::Request<ReqBody>, ResponseBuilder) -> Resp + Send + Sync + 'static,
{
    fn handle(&self, req: http::Request<ReqBody>, resp: ResponseBuilder) -> Resp {
        (self)(req, resp)
    }
}
