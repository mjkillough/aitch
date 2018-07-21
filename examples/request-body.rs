extern crate aitch;
extern crate http;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use aitch::servers::tiny_http::Server;
use aitch::{middlewares, Json, Responder, ResponseBuilder, Result};
use http::Request;

const HTML: &'static str = r#"
<input id="input" />
<button id="button" onclick="javascript: run()">Submit</button>

<script>
async function run() {
    const resp = await fetch('/ajax', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            message: document.getElementById('input').value
        })
    });
    const content = await resp.json();
    console.log(content);
    alert(content.message);
}
</script>
"#;

fn page(_req: Request<()>, mut resp: ResponseBuilder) -> impl Responder {
    resp.header(http::header::CONTENT_TYPE, "text/html")
        .body(HTML.to_owned())
}

#[derive(Serialize, Deserialize)]
struct Message {
    message: String,
}

fn ajax(req: Request<Json<Message>>, mut resp: ResponseBuilder) -> impl Responder {
    let mut json = req.into_body().json();
    json.message += "!";
    resp.header(http::header::CONTENT_TYPE, "application/json")
        .body(Json(json))
}

fn main() -> Result<()> {
    let mut router = middlewares::SimpleRouter::new();
    router.register_handler("/", page);
    router.register_handler("/ajax", ajax);
    let wrapped = middlewares::with_stdout_logging(router);

    let addr = "127.0.0.1:3000".parse()?;
    println!("Listening on http://{}", addr);
    Server::new(addr, wrapped)?.run()
}
