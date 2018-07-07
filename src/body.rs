use bytes::Bytes;
use futures::{future, stream, Future, Stream};

use Error;

pub trait Body
where
    Self: Send + Sized + 'static,
{
    type Future: Future<Item = Self, Error = Error> + Send;

    fn from_stream(stream: BodyStream) -> Self::Future;

    fn into_stream(self) -> BodyStream;
}

impl Body for () {
    type Future = Box<Future<Item = Self, Error = Error> + Send>;

    fn from_stream(_: BodyStream) -> Self::Future {
        Box::new(future::ok(()))
    }

    fn into_stream(self) -> BodyStream {
        empty_body()
    }
}

impl Body for Vec<u8> {
    type Future = Box<Future<Item = Self, Error = Error> + Send>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        Box::new(stream.concat2().map(|bytes| bytes.to_vec()))
    }

    fn into_stream(self) -> BodyStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

impl Body for String {
    type Future = Box<Future<Item = Self, Error = Error> + Send>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        let fut = stream.concat2().and_then(|bytes| {
            let vec = bytes.to_vec();
            let string = String::from_utf8(vec)?;
            Ok(string)
        });
        Box::new(fut)
    }

    fn into_stream(self) -> BodyStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

pub type BodyStream = Box<Stream<Item = Bytes, Error = Error> + Send>;

impl Body for BodyStream {
    type Future = Box<Future<Item = Self, Error = Error> + Send>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        Box::new(future::ok(stream))
    }

    fn into_stream(self) -> BodyStream {
        self
    }
}

pub fn empty_body() -> BodyStream {
    Box::new(stream::once(Ok(Bytes::new())))
}
