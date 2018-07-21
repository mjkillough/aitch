//! Handlers which serve static files (such as assets) from disk.
//!
//! This module provides two handlers, **for use in development only**:
//!
//!  - [`static_file_handler(file_path)`]: serves a file from disk in response to any HTTP request.
//!  - [`static_files_handler(directory_path)`]: serves files from a directory, based on the path of
//!    incoming HTTP requests.
//!
//! [`static_file_handler(file_path)`]: fn.static_file_handler.html
//! [`static_files_handler(directory_path)`]: fn.static_files_handler.html
//!
//! # MIME types
//!
//! By default these handlers will use the [`mime_guess` crate] in order to guess the MIME type of
//! the file being served (based on file extension only), and will use this to set the
//! `Content-Type` header in the response.
//!
//! This feature can be disabled by the `mime_guess` feature on the aitch crate, which removes the
//! dependency on `mime_guess`. If the feature is disabled, all responses will be served with
//! `Content-Type: application/octet-stream`.
//!
//! [`mime_guess` crate]: https://github.com/abonander/mime_guess
//!
//! # I/O errors
//!
//! Any I/O errors that occur while reading files from disk (including file or directory not found)
//! are silently returned as HTTP 404 responses.
//!
//! # Security
//!
//! These handlers aim to be secure and protect against directory traversal attacks. However, it is
//! recommended that these handlers only be used in development, and that in any production system,
//! static assets are served by a separate server, such as nginx or Apache.
//!

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use http;
#[cfg(feature = "mime_guess")]
use mime_guess;

use middlewares;
use {Error, Handler, ResponseBuilder, Result};

/// Serves files from a directory, based on the path of incoming HTTP requests.
///
/// A request of `GET /path/to/file.html` will be translated to `root_dir/path/to/file.html`, and
/// the resulting file returned in the response.
///
/// Any errors that occur while reading the file from disk (including file not found) result in an
/// empty HTTP 404 response.
///
/// See the [module level] documentation for details of the MIME types used to serve responses.
///
/// # Errors
///
/// This function checks the directory exists when called, and attempts to canoncicalize the path.
/// If this fails for any reason (such as directory not found), it returns an error.
///
/// # Security
///
/// This handler is intended **for development use only**. See the [module level] documentation for
/// more details.
///
/// [module level]: ./index.html#security
pub fn static_files_handler<P>(root_path: P) -> Result<impl Handler<()>>
where
    P: AsRef<Path> + Clone + Send + Sync + 'static,
{
    let root_path = root_path.as_ref().to_path_buf().canonicalize()?;

    let handler = middlewares::with_context(root_path, |root_path, req, resp| {
        let path = match_file(root_path, req.uri().path())?;
        serve_file(path, resp)
    });
    let handler = middlewares::with_error_handling(handler, handle_io_errors);

    Ok(handler)
}

fn handle_io_errors(error: Error) -> Result<http::Response<String>> {
    match error.downcast::<io::Error>() {
        Ok(_) => {
            let resp = http::Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .body("404 - not found".to_owned())?;
            Ok(resp)
        }
        Err(error) => Err(error),
    }
}

fn match_file(root: PathBuf, path: &str) -> Result<impl AsRef<Path>> {
    let path = Path::new(path).strip_prefix("/")?;
    let potential = root.join(path).canonicalize()?;

    if !potential.starts_with(root) {
        return Err(io::Error::from(io::ErrorKind::NotFound).into());
    }

    Ok(potential)
}

/// Serves a file from disk in response to any HTTP request.
///
/// Any errors that occur while reading the file from disk (including file not found) result in an
/// empty HTTP 404 response.
///
/// See the [module level] documentation for details of the MIME types used to serve responses.
///
/// # Security
///
/// This handler is intended **for development use only**. See the [module level] documentation for
/// more details.
///
/// [module level]: ./index.html#security
pub fn static_file_handler<P>(path: P) -> impl Handler<Vec<u8>>
where
    P: AsRef<Path> + Clone + Send + Sync + 'static,
{
    let handler = middlewares::with_context(path, |path, _, resp| serve_file(path, resp));
    middlewares::with_error_handling(handler, handle_io_errors)
}

fn serve_file<P>(path: P, mut resp: ResponseBuilder) -> Result<http::Response<Vec<u8>>>
where
    P: AsRef<Path>,
{
    let content = read_file(&path)?;
    let resp = resp.header(http::header::CONTENT_TYPE, mime_type(path).as_str())
        .body(content)?;
    Ok(resp)
}

fn read_file<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(feature = "mime_guess")]
fn mime_type<P>(path: P) -> String
where
    P: AsRef<Path>,
{
    let mime = mime_guess::guess_mime_type(path);
    format!("{}", mime)
}

#[cfg(not(feature = "mime_guess"))]
fn mime_type<P>(_: P) -> String
where
    P: AsRef<Path>,
{
    "application/octet-stream".to_owned()
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::match_file;

    #[test]
    fn match_file_in_directory() {
        let root = Path::new("examples").to_path_buf().canonicalize().unwrap();
        let path = match_file(root.clone(), "/static-files").unwrap();
        let expected = root.join("static-files");
        assert_eq!(path.as_ref().to_str().unwrap(), expected.to_str().unwrap());
    }

    #[test]
    fn match_file_directory_traversal_attack() {
        let path = Path::new("examples").to_path_buf().canonicalize().unwrap();
        let result = match_file(path, "../Cargo.toml");
        assert!(result.is_err());
    }
}
