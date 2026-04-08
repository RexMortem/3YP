use std::panic::{self, AssertUnwindSafe};

use crate::interpreter::run_to_html;
use crate::parser::parse;

const HTML: &str = include_str!("../static/index.html");
const DOCS_HTML: &str = include_str!("../static/documentation.html");
const FEEDBACK_HTML: &str = include_str!("../static/feedback.html");
const CSS: &str = include_str!("../static/style.css");
const JS: &str = include_str!("../static/playground.js");
const NAVBAR_JS: &str = include_str!("../static/navbar.js");
const LOGO_PNG: &[u8] = include_bytes!("../static/yappl_logo.png");

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

            (tiny_http::Method::Get, "/navbar.js") => {
                let response = tiny_http::Response::from_string(NAVBAR_JS).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "text/javascript; charset=utf-8")
                        .unwrap(),
                );
                let _ = request.respond(response);
            }

            (tiny_http::Method::Get, "/documentation.html") => {
                let response = tiny_http::Response::from_string(DOCS_HTML).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "text/html; charset=utf-8")
                        .unwrap(),
                );
                let _ = request.respond(response);
            }

            (tiny_http::Method::Get, "/feedback.html") => {
                let response = tiny_http::Response::from_string(FEEDBACK_HTML).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "text/html; charset=utf-8")
                        .unwrap(),
                );
                let _ = request.respond(response);
            }

            (tiny_http::Method::Get, "/yappl_logo.png") => {
                let response = tiny_http::Response::from_data(LOGO_PNG).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "image/png").unwrap(),
                );
                let _ = request.respond(response);
            }

            (tiny_http::Method::Post, "/run") => {
                let mut code = String::new();
                let _ = request.as_reader().read_to_string(&mut code);

                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    let stmts = parse(&code);
                    run_to_html(&stmts)
                }));

                let (status, content_type, body) = match result {
                    Ok(html) => (200u16, "text/html; charset=utf-8", html),
                    Err(e) => {
                        let msg = e
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "Unknown error".to_string());
                        // Return plain-text error so the JS can detect it via
                        // Content-Type and display it differently.
                        (500, "text/plain; charset=utf-8", format!("Error: {}", msg))
                    }
                };

                let response = tiny_http::Response::from_string(body)
                    .with_status_code(status)
                    .with_header(
                        tiny_http::Header::from_bytes("Content-Type", content_type).unwrap(),
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
