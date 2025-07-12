use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::process::ExitCode;
use std::result::Result;
use xml::common::{Position, TextPosition};
use xml::reader::{EventReader, XmlEvent};
use std::io::{BufReader, BufWriter};

mod model;
use model::*;

mod server;

fn read_entire_xml_file(file_path: &Path) -> Result<String, ()> {
    let file = File::open(file_path).map_err(|err| {
        eprintln!("ERROR: could not open {}: {err}", file_path.display());
    })?;
    let er = EventReader::new(BufReader::new(file));

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

    serde_json::to_writer(BufWriter::new(index_file), &tf_index).map_err(|err| {
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

            let prompt = args
                .next()
                .ok_or_else(|| {
                    usage(&program);
                    eprintln!("ERROR: no search query is provided for {subcommand} subcommand");
                })?
                .chars()
                .collect::<Vec<_>>();

            let index_file = File::open(&index_path).map_err(|err| {
                eprintln!("ERROR: could not open index file {index_path}: {err}");
            })?;

            let tf_index: TermFreqIndex = serde_json::from_reader(index_file).map_err(|err| {
                eprintln!("ERROR: could not parse index file {index_path}: {err}");
            })?;

            for (path, rank) in search_query(&tf_index, &prompt).iter().take(20) {
                println!("Found match: {} (rank: {})", path.display(), rank);
            }

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
            return server::start(&address, &tf_index);
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
