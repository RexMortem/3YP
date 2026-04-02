use std::panic::{self, AssertUnwindSafe};

use crate::interpreter::run_to_string;
use crate::parser::parse;

const HTML: &str = include_str!("../static/index.html");
const CSS: &str = include_str!("../static/style.css");
const JS: &str = include_str!("../static/playground.js");

pub fn serve(port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let server = tiny_http::Server::http(&addr)
        .unwrap_or_else(|e| panic!("Failed to start server on {}: {}", addr, e));

    println!("Web playground running at http://localhost:{}", port);

    for mut request in server.incoming_requests() {
        let method = request.method().clone();
        let url = request.url().to_string();

        // all the routes we need for AJAX reqs
        match (method, url.as_str()) {
            (tiny_http::Method::Get, "/") => {
                let response = tiny_http::Response::from_string(HTML).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "text/html; charset=utf-8")
                        .unwrap(),
                );
                let _ = request.respond(response);
            }

            (tiny_http::Method::Get, "/style.css") => {
                let response = tiny_http::Response::from_string(CSS).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "text/css; charset=utf-8")
                        .unwrap(),
                );
                let _ = request.respond(response);
            }

            (tiny_http::Method::Get, "/playground.js") => {
                let response = tiny_http::Response::from_string(JS).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "text/javascript; charset=utf-8")
                        .unwrap(),
                );
                let _ = request.respond(response);
            }

            (tiny_http::Method::Post, "/run") => {
                let mut code = String::new();
                let _ = request.as_reader().read_to_string(&mut code);

                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    let stmts = parse(&code);
                    run_to_string(&stmts)
                }));

                let (status, body) = match result {
                    Ok(output) => (200u16, output),
                    Err(e) => {
                        let msg = e
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "Unknown error".to_string());
                        (500, format!("Error: {}", msg))
                    }
                };

                let response = tiny_http::Response::from_string(body)
                    .with_status_code(status)
                    .with_header(
                        tiny_http::Header::from_bytes("Content-Type", "text/plain; charset=utf-8")
                            .unwrap(),
                    );
                let _ = request.respond(response);
            }

            _ => {
                let response =
                    tiny_http::Response::from_string("Not Found").with_status_code(404u16);
                let _ = request.respond(response);
            }
        }
    }
}
