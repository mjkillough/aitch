use bytes::Bytes;
use futures::{self, future, stream, Future, Stream};

use {Error, Result};

/// A trait defining the types that can be used in HTTP requests and responses.
///
/// The [`http::Request<B>`] and [`http::Response<B>`] types from the [`http` crate] do not restrict
/// the type of the body used in requests and bodies.
///
/// [`http::Request<B>`]: https://docs.rs/http/0.1.7/http/request/struct.Request.html
/// [`http::Response<B>`]: https://docs.rs/http/0.1.7/http/response/struct.Response.html
/// [`http` crate]: https://github.com/hyperium/http
///
/// This type is used by `aitch` to place some contraints on the body types used, so that handlers,
/// middlewares and servers can make assumptions about how to deserialize it from the raw HTTP
/// request, and how they should be serialized to the raw HTTP responses.
///
/// The trait aims to describe both non-streaming and streaming bodies, allowing it to be used both
/// handlers that process the request asnchronously and those that do so synchronously.
///
/// Types implementing this trait must provide both a way to construct themselves from
/// [`BodyStream`], and a way to construct a [`BodyStream`] from themselves. This allows the same
/// types to be used in both request and response bodies, and to allow multiple layers of
/// handlers/middlewares to interpret a request's body in different ways.
///
/// [`BodyStream`]: type.BodyStream.html
///
/// # Empty Request/Response Bodies
///
/// For requests/responses where the body is not important (such as `GET` HTTP requests, or error
/// responses), the unit type, `()` may be used to denote an empty body.
///
/// # Example
///
/// An implementation of `Body` for the [`bytes::Bytes`] type (which is provided by aitch):
///
/// [`bytes::Bytes`]: http://carllerche.github.io/bytes/bytes/struct.Bytes.html
///
/// ```
/// extern crate aitch;
/// extern crate bytes;
/// extern crate futures;
///
/// use aitch::{Body, BodyStream};
/// use bytes::Bytes;
/// use futures::{stream, Future, Stream};
///
/// // Define a wrapper, to allow implementing for foreign type.
/// struct BytesWrapper(Bytes);
///
/// impl Body for BytesWrapper {
///     type Future = Box<Future<Item = Self, Error = aitch::Error> + Send>;
///
///     fn from_stream(stream: BodyStream) -> Self::Future {
///         Box::new(stream.concat2().map(BytesWrapper))
///     }
///
///     fn into_stream(self) -> BodyStream {
///         Box::new(stream::once(Ok(self.0)))
///     }
/// }
/// ```
pub trait Body
where
    Self: Send + Sized + 'static,
{
    type Future: Future<Item = Self, Error = Error> + Send;

    /// Consume a [`BodyStream`] and return a future which resolves to `Self`.
    ///
    /// [`BodyStream`]: type.BodyStream.html
    fn from_stream(stream: BodyStream) -> Self::Future;

    /// Consume `Self` and returns a [`BodyStream`].
    ///
    /// [`BodyStream`]: type.BodyStream.html
    fn into_stream(self) -> BodyStream;

    /// Convenience function to convert from one type implementing `Body` to another.
    fn into_body<OtherBody: Body>(self) -> OtherBody::Future {
        OtherBody::from_stream(self.into_stream())
    }
}

impl Body for () {
    type Future = future::FutureResult<(), Error>;

    fn from_stream(_: BodyStream) -> Self::Future {
        future::ok(())
    }

    fn into_stream(self) -> BodyStream {
        empty_body()
    }
}

impl Body for Bytes {
    type Future = stream::Concat2<BodyStream>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        stream.concat2()
    }

    fn into_stream(self) -> BodyStream {
        Box::new(stream::once(Ok(self)))
    }
}

type VecFuture = future::Map<stream::Concat2<BodyStream>, fn(Bytes) -> Vec<u8>>;

impl Body for Vec<u8> {
    type Future = VecFuture;

    fn from_stream(stream: BodyStream) -> Self::Future {
        stream.concat2().map(|bytes| bytes.to_vec())
    }

    fn into_stream(self) -> BodyStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

type StringFuture =
    futures::AndThen<stream::Concat2<BodyStream>, Result<String>, fn(Bytes) -> Result<String>>;

impl Body for String {
    type Future = StringFuture;

    fn from_stream(stream: BodyStream) -> Self::Future {
        stream.concat2().and_then(|bytes| {
            let vec = bytes.to_vec();
            let string = String::from_utf8(vec)?;
            Ok(string)
        })
    }

    fn into_stream(self) -> BodyStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

/// A streaming body, which can be used in a HTTP request or response.
///
/// This is a simple type alias for a boxed [`futures::Stream`], which yields chunks of
/// [`bytes::Bytes`], representing a streaming HTTP request/response's body.
///
/// All types that can be used as a HTTP request/response's body in aitch handlers can be converted
/// to/from this type using the [`Body`] trait.
///
/// [`futures::Stream`]: https://docs.rs/futures/0.1.23/futures/future/trait.Stream.html
/// [`bytes::Bytes`]: http://carllerche.github.io/bytes/bytes/struct.Bytes.html
/// [`Body`]: trait.Body.html
///
/// Most applications will not need to use this type directly, but will instead make use of it through the types for which [`Body`] is implemented (such as `()`, `String`, `Vec<u8>`).
pub type BodyStream = Box<Stream<Item = Bytes, Error = Error> + Send>;

impl Body for BodyStream {
    type Future = future::FutureResult<BodyStream, Error>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        future::ok(stream)
    }

    fn into_stream(self) -> BodyStream {
        self
    }
}

fn empty_body() -> BodyStream {
    Box::new(stream::once(Ok(Bytes::new())))
}
