use http;

use {Body, Responder, ResponseBuilder};

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
    ReqBody: Body + 'static,
    Resp: Responder + 'static,
{
    type Resp = Resp;

    fn handle(&self, req: http::Request<ReqBody>, resp: ResponseBuilder) -> Resp {
        (self)(req, resp)
    }
}
