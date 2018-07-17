mod router;

use futures::Future;
use http;

use {Body, Error, Handler, Responder, ResponseBuilder};

pub use self::router::SimpleRouter;

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

pub fn with_error_handling<B, F, R>(handler: impl Handler<B>, func: F) -> impl Handler<B>
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

pub fn with_context<Ctx, Func, ReqBody, Resp>(ctx: Ctx, handler: Func) -> impl Handler<ReqBody>
where
    Ctx: Clone + Send + Sync + 'static,
    Func: Fn(Ctx, http::Request<ReqBody>, ResponseBuilder) -> Resp + Send + Sync + 'static,
    ReqBody: Body,
    Resp: Responder,
{
    move |req, resp| handler(ctx.clone(), req, resp)
}
