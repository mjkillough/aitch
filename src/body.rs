use bytes::Bytes;
use futures::{self, future, stream, Future, Stream};

use {Error, Result};

pub trait Body
where
    Self: Send + Sized + 'static,
{
    type Future: Future<Item = Self, Error = Error> + Send;

    fn from_stream(stream: BodyStream) -> Self::Future;

    fn into_stream(self) -> BodyStream;
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

impl Body for Vec<u8> {
    type Future = future::Map<stream::Concat2<BodyStream>, fn(Bytes) -> Vec<u8>>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        stream.concat2().map(|bytes| bytes.to_vec())
    }

    fn into_stream(self) -> BodyStream {
        let bytes = Bytes::from(self);
        Box::new(stream::once(Ok(bytes)))
    }
}

impl Body for String {
    type Future =
        futures::AndThen<stream::Concat2<BodyStream>, Result<String>, fn(Bytes) -> Result<String>>;

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

pub fn empty_body() -> BodyStream {
    Box::new(stream::once(Ok(Bytes::new())))
}
