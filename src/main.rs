#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_attributes)]
#![allow(unused_associated_type_bounds)]
#![allow(dead_code)]
use model::*;
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::process::ExitCode;
use std::result::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use xml::common::{Position, TextPosition};
use xml::reader::{EventReader, XmlEvent};
mod lexer;
mod model;
mod server;
mod snowball;

fn parse_entire_txt_file(file_path: &Path) -> Result<String, ()> {
    fs::read_to_string(file_path).map_err(|err| {
        eprintln!("ERROR: could not read file {}: {err}", file_path.display());
    })
}

fn parse_entire_xml_file(file_path: &Path) -> Result<String, ()> {
    let file = File::open(file_path).map_err(|err| {
        eprintln!(
            "ERROR: could not open file {file_path}: {err}",
            file_path = file_path.display()
        );
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

fn parse_entire_file_by_extension(file_path: &Path) -> Result<String, ()> {
    let extension = file_path
        .extension()
        .ok_or_else(|| {
            eprintln!(
                "ERROR: can't detect file type of {file_path} without extension",
                file_path = file_path.display()
            );
        })?
        .to_string_lossy();
    match extension.as_ref() {
        "xhtml" | "xml" => parse_entire_xml_file(file_path),
        // TODO: specialized parser for markdown files
        "txt" | "md" => parse_entire_txt_file(file_path),
        _ => {
            eprintln!(
                "ERROR: can't detect file type of {file_path}: unsupported extension {extension}",
                file_path = file_path.display(),
                extension = extension
            );
            Err(())
        }
    }
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

fn add_folder_to_model(
    dir_path: &Path,
    model: Arc<Mutex<InMemoryModel>>,
    skipped: &mut usize,
) -> Result<(), ()> {
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

        let last_modified = file
            .metadata()
            .map_err(|err| {
                eprintln!(
                    "ERROR: could not get metadata for file {file_path}: {err}",
                    file_path = file_path.display()
                );
            })?
            .modified()
            .map_err(|err| {
                eprintln!(
                    "ERROR: could not get last modified time for file {file_path}: {err}",
                    file_path = file_path.display()
                );
            })?;

        if file_type.is_dir() {
            add_folder_to_model(&file_path, Arc::clone(&model), skipped)?;
            continue 'next_file;
        }
        let mut model = model.lock().unwrap();
        if model.requires_reindexing(&file_path, last_modified) {
            let content = match parse_entire_file_by_extension(&file_path) {
                Ok(content) => content.chars().collect::<Vec<_>>(),
                Err(()) => {
                    *skipped += 1;
                    continue 'next_file;
                }
            };

            model.add_document(file_path, last_modified, &content)?;
            println!("⚒️ Indexed.");
        }
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
        /*
        "now" => {
            use std::time::SystemTime;
            let now = SystemTime::now();
            println!("{now:?}");
            Ok(())
        }

        "reindex" => {
            let dir_path = args.next().ok_or_else(|| {
                eprintln!("ERROR: no directory is provided for the {subcommand} subcommand.");
                usage(&program);
            })?;

            if use_sqlite_mode {
                // TODO: Implement sqlite mode for reindexing.
                todo!("Implement sqlite model for reindexing.")
            } else {
                let index_path = "index.json";
                let index_file = File::open(index_path).map_err(|err| {
                    eprintln!("ERROR: could not open index file {index_path}: {err}")
                })?;
                let mut model: InMemoryModel =
                    serde_json::from_reader(index_file).map_err(|err| {
                        eprintln!("ERROR: could not parse index file {index_path}: {err}");
                    })?;
                let mut skipped = 0;
                println!("↪️ reIndexing directory: {dir_path}");
                add_folder_to_model(Path::new(&dir_path), model, &mut skipped)?;
                save_model_as_json(&model, index_path)?;
                println!("✅ reIndexing completed. Skipped {skipped} files.");
            }
            Ok(())
        }

        "index" => {
            let dir_path = args.next().ok_or_else(|| {
                eprintln!("ERROR: no directory is provided for the {subcommand} subcommand.");
                usage(&program);
            })?;

            let mut skipped = 0;

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

                let mut model = Arc::new(Mutex::new(Default::default()));
                // TODO: thing a way through adding Arc<Mutex>> to the sqlite model
                //let mut model = Arc::new(Mutex::new(SqliteModel))
                //println!("📂 Indexing directory: {dir_path}");
                model.begin()?;
                // TODO: implement a special transaction object that implements Drop trait and commits the transaction when it goes out of scope
                add_folder_to_model(Path::new(&dir_path), model, &mut skipped)?;
                model.commit()?;
            } else {
                let index_path = "index.json";
                let model = Arc::new(Mutex::new(Default::default()));
                add_folder_to_model(Path::new(&dir_path), model, &mut skipped)?;
                save_model_as_json(&model, index_path)?;
            }
            println!(
                "✅ Indexing completed. Skipped {skipped} files.",
                skipped = skipped
            );
            Ok(())
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
        */
        "serve" => {
            assert!(!use_sqlite_mode);
            let dir_path = args.next().ok_or_else(|| {
                usage(&program);
                eprintln!("ERROR: no path to index is provided for {subcommand} subcommand");
            })?;

            // TODO: figure out the index_path based on dir_path.
            let index_path = "index.json";
            let address = args.next().unwrap_or("127.0.0.1:6969".to_string());

            let exists = Path::new(&index_path).try_exists().map_err(|err| {
                eprintln!(
                    "ERROR: could not ensure the existance of the {index_path}: {err}",
                    index_path = index_path
                )
            })?;
            let model: Arc<Mutex<InMemoryModel>>;
            if exists {
                let index_file = File::open(&index_path).map_err(|err| {
                    eprintln!(
                        "ERROR: could not open the file {index_path}: {err}",
                        index_path = index_path
                    );
                })?;

                model = Arc::new(Mutex::new(serde_json::from_reader(&index_file).map_err(
                    |err| {
                        eprintln!("ERROR: could not parse index file {index_path}: {err}");
                    },
                )?));
            } else {
                model = Arc::new(Mutex::new(Default::default()));
            }
            {
                let model = Arc::clone(&model);
                thread::spawn(move || {
                    let mut skipped = 0;
                    // TODO: what should be done here in case indexing thread crashes??
                    add_folder_to_model(Path::new(&dir_path), Arc::clone(&model), &mut skipped)
                        .unwrap();
                    let model = model.lock().unwrap();
                    save_model_as_json(&model, &index_path).unwrap();
                });
            }
            server::start(&address, Arc::clone(&model))
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
