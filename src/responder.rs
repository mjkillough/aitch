use futures::{Future, IntoFuture};

use http;
use {Body, BoxedResponse, Error};

pub trait Responder<B>
where
    B: Body,
    Self: Send + 'static,
{
    fn into_response(self) -> BoxedResponse;
}

impl<T, B> Responder<B> for T
where
    T: IntoFuture<Item = http::Response<B>> + Send + 'static,
    T::Error: Into<Error>,
    T::Future: Send + 'static,
    B: Body,
{
    fn into_response(self) -> BoxedResponse {
        let fut = self.into_future()
            .map(|resp| resp.map(|body| body.into_stream()))
            .map_err(|error| error.into());
        Box::new(fut) as BoxedResponse
    }
}
