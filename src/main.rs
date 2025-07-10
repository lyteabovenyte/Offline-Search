use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;
use xml::reader::{EventReader, XmlEvent};

use serde_json::Result;

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
        while let Some(&c) = self.content.first() {
            if c.is_whitespace() {
                self.content = &self.content[1..]; // skip whitespace
            } else {
                break; // stop when a non-whitespace character is found
            }
        }
    }

    // Re-sliceing the content to avoid borrowing issues
    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left(); // ensure we start with non-whitespace content
        if self.content.len() == 0 {
            return None;
        }
        if self.content[0].is_alphanumeric() {
            // Collect characters until a non-alphabetic character is found
            let mut end = 0;
            for (i, &c) in self.content.iter().enumerate() {
                if !c.is_alphanumeric() {
                    end = i;
                    break;
                }
            }
            let token = &self.content[..end];
            self.content = &self.content[end..]; // update content to the remaining part
            Some(token)
        } else {
            // If the first character is not alphanumeric, skip it and continue
            self.content = &self.content[1..];
            self.next_token() // recursively call to find the next token
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn index_document(_doc_content: &str) -> HashMap<String, usize> {
    todo!("Implement the indexing logic here -> a hashmap of terms to their frequencies");
}

fn read_entire_xml_file(file_path: &Path) -> io::Result<String> {
    let file = File::open(file_path)?;
    let er = EventReader::new(file);

    let mut content = String::new(); // buffer to hold the content

    for event in er.into_iter() {
        match event {
            Ok(XmlEvent::Characters(text)) => {
                content.push_str(&text); // append text to the content
                content.push_str(" ");
            }
            Err(e) => {
                eprintln!("ERROR: Error reading XML event: {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, "XML parsing error"));
            }
            _ => {} // ignore other events
        }
    }

    Ok(content)
}

fn main() -> io::Result<()> {
    let index_path = "index.json";
    let index_file = File::open(index_path)?;
    println!("🤓 Reading index file ...");
    let tf_index: TermFreqIndex = serde_json::from_reader(&index_file).unwrap_or_else(|err| {
        eprintln!("ERROR: Serde couldn't open the index file to read from: {err}");
        println!("❗️returning empty index due to error in opening index file to read.");
        return TermFreqIndex::new() // returning an empty index.
    });
    println!("{index_path:?} contains {count:?} files.", count = tf_index.len());
    Ok(())
}

fn main2() -> io::Result<()> {
    let dir_path = "docs.gl/gl4";
    if !fs::metadata(dir_path).is_ok() {
        eprintln!(
            "ERROR: Directory {} does not exist or is not accessible.",
            dir_path
        );
        process::exit(1);
    }
    let dir = fs::read_dir(dir_path).unwrap_or_else(|err| {
        eprintln!("ERROR: Error reading directory {}: {}", dir_path, err);
        process::exit(1);
    });

    let mut tf_index = TermFreqIndex::new();

    for file in dir {
        let file_path = file?.path();
        let content = read_entire_xml_file(&file_path)?.chars().collect::<Vec<_>>();

        let mut tf = TermFreq::new(); // frequency table for terms.
        

        for token in Lexer::new(&content) {
            let term = token
                .iter()
                .map(|c| c.to_ascii_uppercase())
                .collect::<String>();
            *tf.entry(term).or_insert(0) += 1;
        }

        let mut tf_sorted: Vec<_> = tf.iter().collect::<Vec<_>>();
        tf_sorted.sort_by_key(|(_, f)| *f);
        tf_sorted.reverse(); // Sort in descending order of frequency

        println!("⚒️ Indexing {:?} ...", file_path);
        tf_index.insert(file_path.clone(), tf.clone());

        // println!("{} term frequencies:", file_path.display());
        // println!("---------------------");
        // for (t, f) in tf_sorted.iter().take(10) {
        //     println!("\t\t{}: {}", t, f);
        // }
    }

    let index_path = "index.json";
    println!("⚒️ creating index at {index_path:?} ...");
    let index_file = File::create(index_path)?;
    serde_json::to_writer(index_file, &tf_index).unwrap_or_else(|err| {
        eprintln!("ERROR: serder couldn't open the index file to write: {}", err)
    });
    println!("✍️ write completed to the index file.");

    Ok(())
}