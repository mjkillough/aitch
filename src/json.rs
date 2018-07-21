use bytes::{Bytes, IntoBuf};
use futures::{future, stream, Future, Stream};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;

use {Body, BodyStream, Result};

/// A wrapper, indicating a type should be automatically (de)serialized from/to a HTTP
/// request/response body.
///
/// This feature requires a dependency on [`serde`]. This dependency is enabled by default, but can
/// be disabled by removing the optional `serde` feature on the `aitch` crate.
///
/// To use a type with this wrapper, it must implement both [`serde::de::DeserializeOwned`] and
/// [`serde::Serialize`]. This is because `Json<T>` can be used in both request/response bodies, and
/// because chains of handlers may choose to serialize/deserialize the type back into a stream of
/// bytes in order to process the requests. (See the [`Body`] trait for more information).
///
/// [`serde`]: https://serde.rs
/// [`Body`]: trait.Body.html
/// [`serde::de::DeserializeOwned`]: https://docs.serde.rs/serde/de/trait.DeserializeOwned.html
/// [`serde::Serialize`]: https://docs.serde.rs/serde/trait.Serialize.html
///
/// # Examples
///
/// The following example accepts a request with a JSON payload, and uses this to construct a JSON
/// response:
///
/// ```
/// extern crate aitch;
/// extern crate http;
/// extern crate serde;
/// #[macro_use] extern crate serde_derive;
///
/// use aitch::servers::hyper::Server;
/// use aitch::{middlewares, Json, Responder, ResponseBuilder, Result};
/// use http::Request;
///
/// #[derive(Serialize, Deserialize)]
/// struct ReqParameters {
///     message: String,
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct RespBody {
///     message: String,
/// }
///
/// fn handler(req: Request<Json<ReqParameters>>, mut resp: ResponseBuilder) -> impl Responder {
///     let req_params = req.into_body().json();
///     let message = format!("You sent '{}'", req_params.message);
///     let body = RespBody { message };
///
///     resp.header(http::header::CONTENT_TYPE, "application/json")
///         .body(Json(body))
/// }
/// ```
pub struct Json<T>(pub T);

impl<T> Json<T> {
    /// Unwraps the JSON wrapper, returning the underlying type.
    pub fn json(self) -> T {
        self.0
    }
}

type JsonFuture<T> =
    future::AndThen<stream::Concat2<BodyStream>, Result<Json<T>>, fn(Bytes) -> Result<Json<T>>>;

impl<T> Body for Json<T>
where
    T: DeserializeOwned + Serialize + Send + 'static,
{
    type Future = JsonFuture<T>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        stream.concat2().and_then(|bytes| {
            let cursor = bytes.into_buf();
            let json = serde_json::from_reader(cursor)?;
            Ok(Json(json))
        })
    }

    fn into_stream(self) -> BodyStream {
        let stream = stream::once(Ok(self)).and_then(|json| {
            let vec = serde_json::to_vec(&json.0)?;
            Ok(Bytes::from(vec))
        });
        Box::new(stream)
    }
}
