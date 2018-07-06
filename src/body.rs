use bytes::Bytes;
use futures::{stream, Future, Stream};

use {BoxedStream, Error};

pub trait Body
where
    Self: Send + 'static,
{
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static;

    fn into_stream(self) -> BoxedStream;
}

impl Body for Vec<u8> {
    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static,
    {
        Box::new(stream.concat2().map(|bytes| bytes.to_vec()))
    }

    fn into_stream(self) -> BoxedStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}
