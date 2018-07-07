mod router;

use futures::Future;
use http;

use {Body, Handler, Responder, ResponseBuilder};

pub use self::router::SimpleRouter;

pub fn logging_handler<B: Body>(handler: impl Handler<B>) -> impl Handler<B> {
    move |req: http::Request<B>, resp: ResponseBuilder| {
        let method = req.method().clone();
        let uri = req.uri().clone();

        handler.handle(req, resp).into_response().map(move |resp| {
            println!("{} {} {}", method, uri.path(), resp.status());
            resp
        })
    }
}
