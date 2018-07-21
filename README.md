# aitch [![Build Status](https://travis-ci.org/mjkillough/aitch.svg?branch=master)](https://travis-ci.org/mjkillough/aitch) [![Crates.io](https://img.shields.io/crates/v/aitch.svg)](https://crates.io/crates/aitch) [![Docs.rs](https://docs.rs/aitch/badge.svg)](https://docs.rs/aitch/)

aitch is a simple, lightweight toolkit for building HTTP servers in safe, stable Rust.

It builds upon the [`http` crate](https://github.com/hyperium/http), and provides additional types for representing HTTP handlers, bodies and middlewares. It provides both [`hyper`](https://hyper.rs/) and [`tiny_http`](https://github.com/tiny-http/tiny-http) backends for running handlers, but aims to be agnostic of the HTTP server library.

It's inspired by the simplicity (and popularity) of Go's [`net/http` package](https://golang.org/pkg/net/http/), which builds application/middlewares as a series of nested [`Handler`s](https://golang.org/pkg/net/http/#Handler).

aitch takes advantage of Rust's type system in order to make these handlers even easier to work with, without losing the simplicity that makes them so great.

## Example

```rust
extern crate aitch;
extern crate http;

use aitch::servers::hyper::Server;
use aitch::{middlewares, Responder, Result};
use http::Request;

fn handler(_: Request<()>, mut resp: http::response::Builder) -> impl Responder {
    resp.body("Hello, world!".to_owned())
}

fn main() -> Result<()> {
    let wrapped = middlewares::with_stdout_logging(handler);

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped).run()
}
```

This example is provided alongside aitch. To run it, clone this repository and run:

```sh
cargo run --release --example hello-world-sync
```

See also these other examples in the `examples/` directory of this repo:
 - [`json.rs`](examples/json.rs), which uses the optional `Json<T>` type to wrap structs which are automatically (de)serialized in request/response bodies.
 - [`simple-router.rs`](examples/simple-router.rs), which provides a simple request router, modelled after [`net/http`'s `ServeMux`](https://golang.org/pkg/net/http/#ServeMux).
 - [`shared-context.rs`](examples/shared-context.rs), which shows how to use the provided `middlewares::with_context` to inject a struct containing shared resources into an application's HTTP handlers.
 - [`examples/static-files/`](examples/static-files/), which shows how to use the provided `handlers::static_files::*` handlers in order to serve static assets during development.

## Dependencies & Features

aitch aims provide just the types necessary to build HTTP applications with your server technology of choice. It aims to be lightweight in both dependencies and runtime cost, while still being ergonomic to use.

To function, aitch requires a small number of dependencies: `http`, `futures` and `bytes`.

In order to help you be productive quickly, aitch provides a number of optional features, which are currently enabled by default:

 - `server-hyper`: Provides a `Server`, which can run an `aitch::Handler` using the `hyper` web server. [(example)](examples/hello-world-sync.rs)
 - `server-tiny-http`: Provides a `Server`, which can run an `aitch::Handler` using the `tiny_http` web server. [(example)](examples/tiny_http.rs)
 - `json`: Provides a `Json<T>` type, which can wrap any type `T: serde::Deserialize + serde::Serialize`, allowing it to be used in requests and responses: `http::Request<Json<T>>`/`http::Response<Json<T>>`.  [(example)](examples/json.rs)
 - `mime_guess`: Uses the `mime_guess` crate to guess the MIME type of responses returned by the included `handlers::static_files::*` handlers.

These features will probably be split out into separate crates in the near future.

## Is it fast?

It's pretty fast!

When profiling the default `hello-world-sync` example (with logging to `stdout` disabled, using Hyper), with 12 threads and 100 connections, we see ~130,000 req/s on a 2015 13inch MBP:

```
$ wrk --latency -t12 -c100 -d10s http://localhost:3000/
Running 10s test @ http://localhost:3000/
  12 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   837.87us    1.15ms  35.88ms   94.35%
    Req/Sec    11.09k     1.55k   17.44k    75.25%
  Latency Distribution
     50%  630.00us
     75%    1.06ms
     90%    1.61ms
     99%    3.45ms
  1325352 requests in 10.02s, 135.24MB read
Requests/sec: 132296.91
Transfer/sec:     13.50MB
```

## License

MIT
