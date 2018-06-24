use futures::future;
use futures::{self, stream, Future, Stream};
use hyper;

use errors::*;

fn chunk_to_vec(chunk: hyper::Chunk) -> Vec<u8> {
    chunk.to_vec()
}

pub trait HttpBody
where
    Self: Sized,
{
    type Future: Future<Item = Self, Error = Error>;
    type Stream: Stream<Item = Vec<u8>, Error = Error>;

    fn empty() -> Self;
    fn from_hyper_body(hyper::Body) -> Self::Future;
    fn into_stream(self) -> Self::Stream;
}

impl HttpBody for Vec<u8> {
    type Future = futures::MapErr<
        future::Map<stream::Concat2<hyper::Body>, fn(hyper::Chunk) -> Vec<u8>>,
        fn(hyper::Error) -> Error,
    >;
    type Stream = stream::Once<Vec<u8>, Error>;

    fn empty() -> Self {
        Vec::new()
    }

    fn from_hyper_body(body: hyper::Body) -> Self::Future {
        body.concat2()
            .map(chunk_to_vec as fn(hyper::Chunk) -> Vec<u8>)
            .map_err(Error::from)
    }

    fn into_stream(self) -> Self::Stream {
        stream::once(Ok(self))
    }
}
