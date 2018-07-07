use bytes::{Bytes, IntoBuf};
use futures::{future, stream, Future, Stream};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;

use {Body, BodyStream, Result};

pub struct Json<T>(pub T);

impl<T> Body for Json<T>
where
    T: DeserializeOwned + Serialize + Send + 'static,
{
    type Future =
        future::AndThen<stream::Concat2<BodyStream>, Result<Json<T>>, fn(Bytes) -> Result<Json<T>>>;

    fn from_stream(stream: BodyStream) -> Self::Future {
        stream.concat2().and_then(|bytes| {
            let cursor = bytes.into_buf();
            let json = serde_json::from_reader(cursor)?;
            Ok(Json(json))
        })
    }

    fn into_stream(self) -> BodyStream {
        let stream = stream::once(Ok(self)).and_then(|json| {
            let vec = serde_json::to_vec(&json.0)?;
            Ok(Bytes::from(vec))
        });
        Box::new(stream)
    }
}
