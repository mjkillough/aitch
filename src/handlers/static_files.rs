use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use http;

use middlewares;
use {Error, Handler, ResponseBuilder, Result};

pub fn static_files_handler<P>(root: P) -> Result<impl Handler<()>>
where
    P: AsRef<Path> + Clone + Send + Sync + 'static,
{
    let root = root.as_ref().to_path_buf().canonicalize()?;

    let handler = middlewares::with_context(root, |root, req, resp| {
        let path = match_file(root, req.uri().path())?;
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
    let content = read_file(path)?;
    let resp = resp.body(content)?;
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
