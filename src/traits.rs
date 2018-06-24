use std::ops::Deref;

use futures::future::{self, FutureResult, IntoFuture};
use futures::{self, stream, Future, Stream};
use http;
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

// pub trait Responder<Body>
// where
//     Body: HttpBody,
// {
//     type Future: Future<Item = http::Response<Body>, Error = Error>;

//     fn into_future(self) -> Self::Future;
// }

// impl<Body> Responder<Body> for http::Response<Body>
// where
//     Body: HttpBody,
// {
//     type Future = FutureResult<http::Response<Body>, Error>;

//     fn into_future(self) -> Self::Future {
//         future::ok(self)
//     }
// }

// impl<T, Body> Responder<Body> for T
// where
//     T: IntoFuture<Item = http::Response<Body>, Error = Error>,
//     Body: HttpBody,
// {
//     type Future = T;

//     fn into_future(self) -> Self::Future {
//         IntoFuture::into_future(self)
//     }
// }
