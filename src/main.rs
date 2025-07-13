#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]
#![allow(unused_associated_type_bounds)]
#![allow(dead_code)]
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::process::ExitCode;
use std::result::Result;
use xml::common::{Position, TextPosition};
use xml::reader::{EventReader, XmlEvent};
use model::*;

mod model;
mod server;
mod snowball;

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
    let tf_index: TermFreqPerDoc = serde_json::from_reader(index_file).map_err(|err| {
        eprintln!("ERROR: could not parse index file: {err}");
    })?;
    println!(
        "{index_path} contains {count} files",
        count = tf_index.len()
    );
    Ok(())
}

fn save_model_as_json(model: &InMemoryModel, index_path: &str) -> Result<(), ()> {
    println!("🛟 Saving index at {index_path}");
    let index_file = File::create(index_path).map_err(|err| {
        eprintln!("ERROR: could not create the index file at {index_path}: {err}");
    })?;

    serde_json::to_writer(BufWriter::new(index_file), &model).map_err(|err| {
        eprintln!("ERROR: could not serialize index into {index_path:?}: {err}");
    })?;
    Ok(())
}

fn add_folder_to_model(dir_path: &Path, model: &mut dyn Model) -> Result<(), ()> {
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
            add_folder_to_model(&file_path, model)?;
            continue 'next_file;
        }

        println!("⚒️ Indexing {file_path:?}");

        let content = match read_entire_xml_file(&file_path) {
            Ok(content) => content.chars().collect::<Vec<_>>(),
            Err(()) => continue 'next_file,
        };

        model.add_document(file_path, &content)?;
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

    let mut subcommand = None;
    let mut use_sqlite_mode = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--sqlite" => use_sqlite_mode = true,
            _ => {
                subcommand = Some(arg);
                break;
            }
        }
    }

    let subcommand = subcommand.ok_or_else(|| {
        usage(&program);
        println!("ERROR: no subcommand is provided");
    })?;

    match subcommand.as_str() {
        "index" => {
            let dir_path = args.next().ok_or_else(|| {
                eprintln!("ERROR: no directory is provided for the {subcommand} subcommand.");
                usage(&program);
            })?;

            if use_sqlite_mode {
                let index_path = "index.db";
                if let Err(err) = fs::remove_file(index_path) {
                    if err.kind() != std::io::ErrorKind::NotFound {
                        eprintln!(
                            "ERROR: could not remove existing index file {index_path}: {err}"
                        );
                    }
                }
                println!("✅ Removed existing index file {index_path}");

                let mut model = SqliteModel::open(Path::new(index_path))?;
                println!("📂 Indexing directory: {dir_path}");
                model.begin()?;
                add_folder_to_model(Path::new(&dir_path), &mut model)?;
                model.commit()
            } else {
                let index_path = "index.json";
                let mut model = Default::default();
                println!("📂 Indexing directory: {dir_path}");
                add_folder_to_model(Path::new(&dir_path), &mut model)?;
                save_model_as_json(&model, index_path)
            }
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

            if use_sqlite_mode {
                let model = SqliteModel::open(Path::new(&index_path))?;
                println!("🔍 Searching for: {}\n", prompt.iter().collect::<String>());
                for (path, rank) in model.search_query(&prompt)?.iter().take(20) {
                    println!("\t🧩 Found match: {} (rank: {})", path.display(), rank);
                }
            } else {
                let index_file = File::open(&index_path).map_err(|err| {
                    eprintln!("ERROR: could not open index file {index_path}: {err}");
                })?;
                // TODO: should we use BufReader here?
                let model: InMemoryModel = serde_json::from_reader::<_, InMemoryModel>(index_file)
                    .map_err(|err| {
                        eprintln!("ERROR: could not parse index file {index_path}: {err}");
                    })?;
                for (path, rank) in model.search_query(&prompt)?.iter().take(20) {
                    println!("\t🧩 Found match: {} (rank: {})", path.display(), rank);
                }
            }
            println!("✅ Search completed.");
            Ok(())
        }
        "serve" => {
            let index_path = args.next().ok_or_else(|| {
                usage(&program);
                eprintln!("ERROR: no path to index is provided for {subcommand} subcommand");
            })?;

            let address = args.next().unwrap_or("127.0.0.1:6969".to_string());
            if use_sqlite_mode {
                let model = SqliteModel::open(Path::new(&index_path)).map_err(|err| {
                    eprintln!("ERROR: could not open index file {index_path}: {err:?}");
                })?;
                server::start(&address, &model)
            } else {
                let index_file = File::open(&index_path).map_err(|err| {
                    eprintln!("ERROR: could not open index file {index_path}: {err}");
                })?;
                // TODO: should we use BufReader here?
                let model: InMemoryModel = serde_json::from_reader(index_file).map_err(|err| {
                    eprintln!("ERROR: could not parse index file {index_path}: {err}");
                })?;
                server::start(&address, &model)
            }
        }
        _ => {
            println!("ERROR: unknown subcommand {subcommand}");
            Err(())
        }
    }
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
