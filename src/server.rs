use std::fs::File;
use std::str;

use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use super::model::*;

fn serve_404(request: Request) -> Result<(), ()> {
    let response = Response::from_string("404 Not Found")
        .with_status_code(StatusCode(404))
        .with_header(Header::from_bytes("Content-Type", "text/plain; charset=utf-8").unwrap());

    request.respond(response).map_err(|err| {
        eprintln!("ERROR: could not respond to the request: {err}");
    })
}

fn serve_static_file(request: Request, file_path: &str, content_type: &str) -> Result<(), ()> {
    let content_type_header =
        Header::from_bytes("Content-Type", content_type).expect("ERROR: Header is empty");
    // TODO: check if file exists and if it doesn't serve 404
    let file = File::open(file_path).map_err(|err| {
        eprintln!("ERROR: could not serve the file {file_path}: {err}");
    })?;
    let response = Response::from_file(file).with_header(content_type_header);

    request.respond(response).map_err(|err| {
        eprintln!("ERROR: could not serve static file {file_path}: {err}");
    })
}

fn serve_api_search(model: &Model, mut request: Request) -> Result<(), ()> {
    let mut buf = Vec::new();
    request.as_reader().read_to_end(&mut buf).map_err(|err| {
        eprintln!("ERROR: could not read the body of the request: {err}");
    })?;
    let body = str::from_utf8(&buf)
        .map_err(|err| {
            eprintln!("ERROR: could not interpret body as UTF-8 string: {err}");
        })?
        .chars()
        .collect::<Vec<_>>(); // this will help us to use the Lexer.

    println!(
        "🔎 Searching: {body:?}\n💻 INFO: results appear on your browser.",
        body = body.iter().collect::<String>()
    );

    let results = search_query(model, &body);

    let json =
        serde_json::to_string(&results.iter().take(20).collect::<Vec<_>>()).map_err(|err| {
            eprintln!("ERROR: could not serialize results to JSON: {err}");
        })?;

    let content_type_header = Header::from_bytes("Content-Type", "application/json; charset=utf-8")
        .expect("ERROR: Header is empty");

    let response = Response::from_string(&json).with_header(content_type_header);
    request.respond(response).map_err(|err| {
        eprintln!("ERROR: could not respond to the request: {err}");
    })
}

fn serve_request(model: &Model, request: Request) -> Result<(), ()> {
    match (request.method(), request.url()) {
        (Method::Post, "/api/search") => {
            println!(
                "📞 Received Incoming request:  method: {}, url: {}",
                request.method(),
                request.url()
            );
            serve_api_search(model, request)
        }
        (Method::Get, "/index.js") => {
            println!(
                "📞 Received Incoming request:  method: {}, url: {}",
                request.method(),
                request.url()
            );
            serve_static_file(request, "index.js", "application/javascript; charset=utf-8")
        }
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            println!(
                "📞 Received Incoming request:  method: {}, url: {}",
                request.method(),
                request.url()
            );
            serve_static_file(request, "index.html", "text/html; charset=utf-8")
        }
        _ => serve_404(request),
    }
}

pub fn start(address: &str, model: &Model) -> Result<(), ()> {
    let server = Server::http(address).map_err(|err| {
        eprintln!("ERROR: could not start the HTTP server at {address}: {err}");
    })?;

    println!("👂 INFO: Listening at http://{address}");

    for request in server.incoming_requests() {
        serve_request(model, request).ok();
    }

    eprintln!("👋 INFO: Server stopped listening at http://{address}");
    Ok(())
}
