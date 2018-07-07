use bytes::{Bytes, IntoBuf};
use futures::{stream, Future, Stream};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;

use {Body, BodyStream, Error};

pub struct Json<T>(pub T);

impl<T> Body for Json<T>
where
    T: DeserializeOwned + Serialize + Send + 'static,
{
    type Future = Box<Future<Item = Self, Error = Error> + Send>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        let fut = stream.concat2().and_then(|bytes| {
            let cursor = bytes.into_buf();
            let json = serde_json::from_reader(cursor)?;
            Ok(Json(json))
        });
        Box::new(fut)
    }

    fn into_stream(self) -> BodyStream {
        let stream = stream::once(Ok(self)).and_then(|json| {
            let vec = serde_json::to_vec(&json.0)?;
            Ok(Bytes::from(vec))
        });
        Box::new(stream)
    }
}
