use http;
use futures;

use super::{AsyncBody, FutureResponse, SyncBody};


pub trait EmptyBody {
    fn empty() -> Self;
}

impl<T> EmptyBody for T
where
    T: Default,
{
    fn empty() -> T {
        T::default()
    }
}


pub trait FromHttpResponse<Body> {
    fn from_http_response(resp: http::Response<Body>) -> Self;
}

impl FromHttpResponse<AsyncBody> for FutureResponse<AsyncBody> {
    fn from_http_response(resp: http::Response<AsyncBody>) -> Self {
        Box::new(futures::future::ok(resp))
    }
}

impl FromHttpResponse<SyncBody> for http::Response<SyncBody> {
    fn from_http_response(resp: http::Response<SyncBody>) -> Self {
        resp
    }
}


pub trait IntoResponse<Resp> {
    fn into_response(self) -> Resp;
}

impl IntoResponse<http::Response<SyncBody>> for http::Response<SyncBody> {
    fn into_response(self) -> http::Response<SyncBody> {
        self
    }
}

impl IntoResponse<http::Response<SyncBody>> for http::Result<http::Response<SyncBody>> {
    fn into_response(self) -> http::Response<SyncBody> {
        self.unwrap_or_else(|_| {
            http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(SyncBody::empty())
                .unwrap()
        })
    }
}

impl IntoResponse<FutureResponse<AsyncBody>> for FutureResponse<AsyncBody> {
    fn into_response(self) -> FutureResponse<AsyncBody> {
        self
    }
}
