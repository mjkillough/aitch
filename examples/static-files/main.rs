extern crate aitch;
extern crate http;

use aitch::handlers::static_files::static_files_handler;
use aitch::servers::hyper::Server;
use aitch::{middlewares, Result};

fn main() -> Result<()> {
    let handler = static_files_handler("./")?;
    let wrapped = middlewares::with_stdout_logging(handler);

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped).run()
}
