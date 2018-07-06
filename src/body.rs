use bytes::Bytes;
use futures::{future, stream, Future, Stream};

use Error;

pub trait Body
where
    Self: Send + 'static,
{
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static;

    fn into_stream(self) -> BodyStream;
}

impl Body for Vec<u8> {
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static,
    {
        Box::new(stream.concat2().map(|bytes| bytes.to_vec()))
    }

    fn into_stream(self) -> BodyStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

pub type BodyStream = Box<Stream<Item = Bytes, Error = Error> + Send>;

impl Body for BodyStream {
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static,
    {
        // TODO: avoid reboxing something already boxed?
        Box::new(future::ok(Box::new(stream) as BodyStream))
    }

    fn into_stream(self) -> BodyStream {
        self
    }
}
