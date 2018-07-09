use std::io::Cursor;

use bytes::{Bytes, IntoBuf};
use futures::{Future, Stream};
use http;
use tiny_http;

use {BodyStream, Error};

/// Returns a `Future<Item = tiny_http::Response<Cursor<Bytes>>` representing an `http::Response<BodyStream>`.
pub fn as_tiny_http_response(
    resp: http::Response<BodyStream>,
) -> impl Future<Item = tiny_http::Response<Cursor<Bytes>>, Error = Error> {
    let status_code = tiny_http::StatusCode(resp.status().as_u16());

    let headers = resp.headers()
        .iter()
        .map(|(name, value)| {
            let name: &[u8] = name.as_ref();
            let value: &[u8] = value.as_ref();
            tiny_http::Header::from_bytes(name, value).unwrap()
        })
        .collect();

    resp.into_body().concat2().and_then(move |body| {
        Ok(tiny_http::Response::new(
            status_code,
            headers,
            body.into_buf(),
            None,
            None,
        ))
    })
}
