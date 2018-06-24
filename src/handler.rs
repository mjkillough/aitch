use futures::IntoFuture;
use http;

use errors::*;
use traits::HttpBody;
use ResponseBuilder;

pub trait Handler<Body, Resp>
where
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error>,
    Body: HttpBody,
{
    fn handle(&self, &mut http::Request<Body>, ResponseBuilder) -> Resp;
}

impl<Func, Body, Resp> Handler<Body, Resp> for Func
where
    Body: HttpBody,
    Resp: IntoFuture<Item = http::Response<Body>, Error = http::Error>,
    Func: Fn(&mut http::Request<Body>, ResponseBuilder) -> Resp,
{
    fn handle(&self, req: &mut http::Request<Body>, resp: ResponseBuilder) -> Resp {
        (self)(req, resp)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sync_handler_returning_response(
        _req: &mut http::Request<Vec<u8>>,
        mut resp: ResponseBuilder,
    ) -> http::Result<http::Response<Vec<u8>>> {
        resp.body(Vec::<u8>::new())
    }

    fn sync_handler_returning_result(
        _req: &mut http::Request<Vec<u8>>,
        mut resp: ResponseBuilder,
    ) -> http::Result<http::Response<Vec<u8>>> {
        resp.body(Vec::<u8>::new())
    }

    // fn async_handler(
    //     _req: &mut http::Request<AsyncBody>,
    //     mut resp: ResponseBuilder,
    // ) -> FutureResponse<AsyncBody> {
    //     let resp = resp.body(AsyncBody::empty());
    //     Box::new(futures::future::ok(resp))
    // }

    fn request<Body>() -> http::Request<Body>
    where
        Body: HttpBody,
    {
        http::Request::builder().body(Body::empty()).unwrap()
    }

    #[test]
    fn test_sync_handler_func_returning_response() {
        let handler = sync_handler_returning_response;
        let resp = handler
            .handle(&mut request(), ResponseBuilder::new())
            .unwrap();
        assert_eq!(resp.body(), &Vec::new());
    }

    #[test]
    fn test_sync_handler_func_returning_result() {
        let handler = sync_handler_returning_result;
        let resp = handler
            .handle(&mut request(), ResponseBuilder::new())
            .unwrap();
        assert_eq!(resp.body(), &Vec::new());
    }

    #[test]
    fn test_sync_handler_closure() {
        let handler = |req: &mut http::Request<Vec<u8>>, resp: ResponseBuilder| {
            sync_handler_returning_response(req, resp)
        };
        let resp = handler
            .handle(&mut request(), ResponseBuilder::new())
            .unwrap();
        assert_eq!(resp.body(), &Vec::new());
    }

    // #[test]
    // fn test_async_handler() {
    //     let handler = async_handler;
    //     let fut = handler.handle(&mut request(), ResponseBuilder::new());
    //     let resp = fut.wait().unwrap();
    //     let (_, body_stream) = resp.into_parts();
    //     let body = body_stream.concat2().wait().unwrap().to_vec();
    //     assert_eq!(body, vec![],);
    // }
}
