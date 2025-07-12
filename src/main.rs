use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::result::Result;
use xml::common::{Position, TextPosition};
use xml::reader::{EventReader, XmlEvent};

use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

/// TermFreq is the term to frequency table for each file.
type TermFreq = HashMap<String, usize>;

/// TermFreqIndex is the TermFreq for each Doc in the directory
/// each directory caontains multiple files that are each a PathBuf
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        // This function trims the left side of the content until a non-whitespace character is found
        while !self.content.is_empty() && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.chop(n)
    }

    fn next_token(&mut self) -> Option<String> {
        self.trim_left();
        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(
                self.chop_while(|x| x.is_numeric())
                    .iter()
                    .collect::<String>(),
            );
        }

        if self.content[0].is_alphabetic() {
            return Some(
                self.chop_while(|x| x.is_alphanumeric())
                    .iter()
                    .map(|x| x.to_ascii_uppercase())
                    .collect::<String>(),
            );
        }

        Some(self.chop(1).iter().collect::<String>())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn read_entire_xml_file(file_path: &Path) -> Result<String, ()> {
    let file = File::open(file_path).map_err(|err| {
        eprintln!("ERROR: could not open {}: {err}", file_path.display());
    })?;
    let er = EventReader::new(file);

    let mut content = String::new(); // buffer to hold the content

    for event in er.into_iter() {
        let event = event.map_err(|err| {
            let TextPosition { row, column } = err.position();
            let msg = err.msg();
            eprintln!(
                "{file_path}:{row}:{column}: ERROR: {msg}",
                file_path = file_path.display()
            );
        })?;

        if let XmlEvent::Characters(text) = event {
            content.push_str(&text);
            content.push(' ');
        }
    }
    Ok(content)
}

fn check_index(index_path: &str) -> Result<(), ()> {
    let index_file = File::open(index_path).map_err(|err| {
        eprintln!("ERROR: could not open index file: {err}");
    })?;
    println!("🤓 Reading {index_path} index file...");
    let tf_index: TermFreqIndex = serde_json::from_reader(index_file).map_err(|err| {
        eprintln!("ERROR: could not parse index file: {err}");
    })?;
    println!(
        "{index_path} contains {count} files",
        count = tf_index.len()
    );
    Ok(())
}

fn save_tf_index(tf_index: TermFreqIndex, index_path: &str) -> Result<(), ()> {
    println!("🛟 Saving index at {index_path}");
    let index_file = File::create(index_path).map_err(|err| {
        eprintln!("ERROR: could not create the index file at {index_path}: {err}");
    })?;

    serde_json::to_writer(index_file, &tf_index).map_err(|err| {
        eprintln!("ERROR: could not serialize index into {index_path:?}: {err}");
    })?;
    Ok(())
}

fn tf_index_of_folder(dir_path: &Path, tf_index: &mut TermFreqIndex) -> Result<(), ()> {
    let dir = fs::read_dir(dir_path).map_err(|err| {
        eprintln!("ERROR: could not open directory {dir_path:?}: {err}");
    })?;

    'next_file: for file in dir {
        let file = file.map_err(|err| {
            eprintln!(
                "ERROR: could not read next file in directory {dir_path} during indexing: {err}",
                dir_path = dir_path.display()
            );
        })?;

        let file_path = file.path();

        let file_type = file.file_type().map_err(|err| {
            eprintln!("ERROR: could not determine the type of the file {file:?}: {err:?}");
        })?;

        if file_type.is_dir() {
            tf_index_of_folder(&file_path, tf_index)?;
            continue 'next_file;
        }

        println!("⚒️ Indexing {file_path:?}");

        let content = match read_entire_xml_file(&file_path) {
            Ok(content) => content.chars().collect::<Vec<_>>(),
            Err(()) => continue 'next_file,
        };

        let mut tf = TermFreq::new();

        for token in Lexer::new(&content) {
            *tf.entry(token).or_insert(0) += 1;
        }

        tf_index.insert(file_path, tf);
    }
    Ok(())
}

fn usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!(
        "    index <folder>         index the <folder> and save the index to index.json file"
    );
    eprintln!(
        "    search <index-file>     check how many documents are indexed in the file (searching is not implemented yet)"
    );
    eprintln!("    serve <index-file> [address]       start local HTTP server with Web Interface");
}

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

    let file = File::open(file_path).map_err(|err| {
        eprintln!("ERROR: could not serve the file {file_path}: {err}");
    })?;
    let response = Response::from_file(file).with_header(content_type_header);

    request.respond(response).map_err(|err| {
        eprintln!("ERROR: could not serve static file {file_path}: {err}");
    })
}

/// Returns the total frequency of the term `t` in the document frequency index `d`.
/// It sums up the term frequencies across all documents in the index.
/// If the term is not found in a document, it contributes 0 to the sum.
fn tf(t: &str, d: &TermFreq) -> f32 {
    d.get(t).cloned().unwrap_or(0) as f32 / d.iter().map(|(_, v)| *v).sum::<usize>() as f32
}

/// Returns the inverse document frequency (IDF) of the term `t` in the document frequency index `d`.
/// It calculates the logarithm of the ratio of the total number of documents to the number of documents containing the term.
/// If the term is not found in any document, it returns 0.
/// The IDF is a measure of how important a term is in the context of the entire document collection.
fn idf(t: &str, d: &TermFreqIndex) -> f32 {
    let n: f32 = d.len() as f32;
    let m: f32 = 1f32 + d.values().filter(|tf| tf.contains_key(t)).count().max(1) as f32;
    return (n / m).log10();
}

fn serve_api_search(tf_index: &TermFreqIndex, mut request: Request) -> Result<(), ()> {
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
        "🔎 Searching: {body:?}\n",
        body = body.iter().collect::<String>()
    );

    let mut results: Vec<(&Path, f32)> = Vec::new();
    for (path, tf_table) in tf_index {
        let mut rank = 0.0;
        for token in Lexer::new(&body) {
            rank += tf(&token, tf_table) * idf(&token, tf_index);
        }
        results.push((path, rank));
    }

    results.sort_by(|(_, rank1), (_, rank2)| rank2.partial_cmp(rank1).unwrap());

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

fn serve_request(tf_index: &TermFreqIndex, request: Request) -> Result<(), ()> {
    match (request.method(), request.url()) {
        (Method::Post, "/api/search") => {
            println!(
                "📞 Received Incoming request:  method: {}, url: {}",
                request.method(),
                request.url()
            );
            serve_api_search(&tf_index, request)
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

fn entry() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("path to program is provided");

    let subcommand = args.next().ok_or_else(|| {
        usage(&program);
        println!("ERROR: no subcommand is provided");
    })?;

    match subcommand.as_str() {
        "index" => {
            let dir_path = args.next().ok_or_else(|| {
                eprintln!("ERROR: no directory is provided for the {subcommand} subcommand.");
                usage(&program);
            })?;

            let mut tf_index = TermFreqIndex::new();
            tf_index_of_folder(Path::new(&dir_path), &mut tf_index)?;
            save_tf_index(tf_index, "index.json")?;
        }
        "search" => {
            let index_path = args.next().ok_or_else(|| {
                eprintln!("ERROR: no path to index is provided for {subcommand} subcommand");
                usage(&program);
            })?;

            check_index(&index_path)?;
        }
        "serve" => {
            let index_path = args.next().ok_or_else(|| {
                usage(&program);
                eprintln!("ERROR: no path to index is provided for {subcommand} subcommand");
            })?;

            let index_file = File::open(&index_path).map_err(|err| {
                eprintln!("ERROR: could not open index file {index_path}: {err}");
            })?;

            let tf_index: TermFreqIndex = serde_json::from_reader(index_file).map_err(|err| {
                eprintln!("ERROR: could not parse index file {index_path}: {err}");
            })?;

            let address = args.next().unwrap_or("127.0.0.1:6969".to_string());
            let server = Server::http(&address).map_err(|err| {
                eprintln!("ERROR: could not start the HTTP server at {address}: {err}");
            })?;

            println!("👂 INFO: Listening at http://{address}");

            for request in server.incoming_requests() {
                serve_request(&tf_index, request).ok();
            }
        }
        _ => {
            println!("ERROR: unknown subcommand {subcommand}");
            return Err(());
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
