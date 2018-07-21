use futures::{Future, IntoFuture};

use http;
use {Body, BoxedResponse, Error};

/// A `Responder` is any type which is returned from a request [`Handler`].
///
/// This trait exists so that the ergonomics of writing [`Handler`] functions are nicer, as they can
/// return an `impl Responder` type, which is automatically inferred based on the body of the
/// function.
///
/// The `Responder` trait is automatically implemented for any type which implements:
///
/// ```ignore
/// IntoFuture<Item = http::Response<impl Body>, Error: impl Into<aitch::Error>>
/// ```
///
/// Importantly, this means it is implemented for the following types:
///
///  - `http::Result<http::Reponse<impl Body>>`, which is the result of calling
///    `http::response::Builder::body(...)` inside of a [`Handler`].
///  - A [`Future`] which eventually yields a `http::Response<impl Body>`.
///
/// As this implementation is done through a blanket impl for any `T`, it is unlikely that
/// downstream crates will be able to implement `Responder` for any custom types. Downstream crates
/// should instead ensure that their types implement `IntoFuture` (with the appropriate
/// constraints), to ensure that they can make use of the `Responder` impl provided by aitch.
///
/// [`Handler`]: trait.Handler.html
/// [`Future`]: https://docs.rs/futures/0.1.23/futures/future/trait.Future.html
pub trait Responder: Send + 'static {
    type Body: Body;

    /// Converts the response into a [`BoxedResponse`].
    ///
    /// [`BoxedResponse`]: type.BoxedResponse.html
    fn into_response(self) -> BoxedResponse;
}

// TODO: See if we can get rid of the `BoxedResponse` and instead return a static type. Simple
// experiments showed this to be somehow slower?
impl<T, B> Responder for T
where
    T: IntoFuture<Item = http::Response<B>> + Send + 'static,
    T::Error: Into<Error>,
    T::Future: Send + 'static,
    B: Body,
{
    type Body = B;

    fn into_response(self) -> BoxedResponse {
        let fut = self.into_future()
            .map(|resp| resp.map(|body| body.into_stream()))
            .map_err(|error| error.into());
        Box::new(fut) as BoxedResponse
    }
}
