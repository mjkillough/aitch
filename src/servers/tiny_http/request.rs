use std::str::FromStr;

use bytes::Bytes;
use futures::stream;
use http;
use tiny_http;

use {BodyStream, Result};

/// Creates a `http::Request<BodyStream>` representing a `tiny_http::Request`.
pub fn as_http_request(req: &mut tiny_http::Request) -> Result<http::Request<BodyStream>> {
    let method = map_method(req.method())?;
    let uri: http::Uri = http::HttpTryFrom::try_from(req.url())?;
    let version = map_version(req.http_version())?;

    let mut builder = http::request::Builder::new();
    builder.method(method).uri(uri).version(version);

    for header in req.headers() {
        let (name, value) = map_header(header)?;
        builder.header(name, value);
    }

    let body = read_body(req)?;

    Ok(builder.body(body)?)
}

fn map_method(method: &tiny_http::Method) -> Result<http::Method> {
    let mapped = match method {
        tiny_http::Method::Get => http::Method::GET,
        tiny_http::Method::Head => http::Method::HEAD,
        tiny_http::Method::Post => http::Method::POST,
        tiny_http::Method::Put => http::Method::PUT,
        tiny_http::Method::Delete => http::Method::DELETE,
        tiny_http::Method::Connect => http::Method::CONNECT,
        tiny_http::Method::Options => http::Method::OPTIONS,
        tiny_http::Method::Trace => http::Method::TRACE,
        tiny_http::Method::Patch => http::Method::PATCH,
        tiny_http::Method::NonStandard(ascii_string) => {
            http::Method::from_str(ascii_string.as_str())?
        }
    };
    Ok(mapped)
}

fn map_version(version: &tiny_http::HTTPVersion) -> Result<http::Version> {
    let (major, minor) = (version.0, version.1);
    let version = match (major, minor) {
        (0, 9) => http::Version::HTTP_09,
        (1, 0) => http::Version::HTTP_10,
        (1, 1) => http::Version::HTTP_11,
        (2, 0) => http::Version::HTTP_2,
        // TODO: Return some kind of error.
        _ => panic!("Unknown HTTP Version: ({}, {})", major, minor),
    };
    Ok(version)
}

fn map_header(
    header: &tiny_http::Header,
) -> Result<(http::header::HeaderName, http::header::HeaderValue)> {
    let name = http::header::HeaderName::from_str(header.field.as_str().as_str())?;
    let value = http::header::HeaderValue::from_str(header.value.as_str())?;
    Ok((name, value))
}

fn read_body(req: &mut tiny_http::Request) -> Result<BodyStream> {
    let content_length = req.body_length().unwrap_or(0);

    let mut buf = Vec::with_capacity(content_length);
    req.as_reader().read_to_end(&mut buf)?;

    let bytes = Bytes::from(buf);
    let stream = stream::once(Ok(bytes));
    Ok(Box::new(stream))
}
