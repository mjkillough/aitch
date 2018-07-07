use bytes::Bytes;
use futures::{future, stream, Future, Stream};

use Error;

pub trait Body
where
    Self: Send + 'static,
{
    fn from_stream(stream: BodyStream) -> Box<Future<Item = Self, Error = Error> + Send>;

    fn into_stream(self) -> BodyStream;
}

impl Body for () {
    fn from_stream(_: BodyStream) -> Box<Future<Item = Self, Error = Error> + Send> {
        Box::new(future::ok(()))
    }

    fn into_stream(self) -> BodyStream {
        empty_body()
    }
}

impl Body for Vec<u8> {
    fn from_stream(stream: BodyStream) -> Box<Future<Item = Self, Error = Error> + Send> {
        Box::new(stream.concat2().map(|bytes| bytes.to_vec()))
    }

    fn into_stream(self) -> BodyStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

impl Body for String {
    fn from_stream(stream: BodyStream) -> Box<Future<Item = Self, Error = Error> + Send> {
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
    fn from_stream(stream: BodyStream) -> Box<Future<Item = Self, Error = Error> + Send> {
        Box::new(future::ok(stream))
    }

    fn into_stream(self) -> BodyStream {
        self
    }
}

pub fn empty_body() -> BodyStream {
    Box::new(stream::once(Ok(Bytes::new())))
}
