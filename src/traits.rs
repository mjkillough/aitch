use bytes::Bytes;
use futures::{stream, Future, Stream};
use hyper;

use errors::*;

pub trait Body
where
    Self: Sized,
{
    fn empty() -> Self;

    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static;

    fn into_stream(self) -> Box<Stream<Item = Bytes, Error = Error> + Send>;
}

impl Body for Vec<u8> {
    fn empty() -> Self {
        Vec::new()
    }

    fn from_stream<S>(stream: S) -> Box<Future<Item = Self, Error = Error> + Send>
    where
        S: Stream<Item = Bytes, Error = Error> + Send + 'static,
    {
        Box::new(stream.concat2().map(|bytes| bytes.to_vec()))
    }

    fn into_stream(self) -> Box<Stream<Item = Bytes, Error = Error> + Send> {
        Box::new(stream::once(Ok(self.into())))
    }
}
