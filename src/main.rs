use std::fs::{self, File};
use std::io::{self, Read};
use std::process;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;

struct Lexer<'a> {
    content: &'a [char] ,
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn next_token(&mut self) -> Option<String> {
        // Implement the logic to return the next token from the content
        // For now, we will return None to indicate no more tokens
        None
    }
}

fn index_document(_doc_content: &str) -> HashMap<String, usize> {
    todo!("Implement the indexing logic here -> a hashmap of terms to their frequencies");
}

fn read_entire_xml_file<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let file = File::open(file_path)?;
    let er = EventReader::new(file);

    let mut content = String::new(); // buffer to hold the content

    for event in er.into_iter() {
        match event {
            Ok(XmlEvent::Characters(text)) => {
                content.push_str(&text); // append text to the content
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

fn main() -> io::Result<()>{
    let dir_path = "docs.gl/gl4";
    if !fs::metadata(dir_path).is_ok() {
        eprintln!("ERROR: Directory {} does not exist or is not accessible.", dir_path);
        process::exit(1);
    }
    let dir = fs::read_dir(dir_path).unwrap_or_else(|err| {
        eprintln!("ERROR: Error reading directory {}: {}", dir_path, err);
        process::exit(1);
    });

    for file in dir {
        let file_path = file?.path();
        let content = read_entire_xml_file(file_path.to_str().unwrap()).unwrap_or_else(|err| {
            eprintln!("ERROR: Error in reading file {:?}: {}", file_path, err);
            process::exit(1);
        });
        // Here you would call index_document(content) to index the content

        // println!("the size of {}: {}", file_path.display(), content.len());
    }

    let example = read_entire_xml_file("docs.gl/gl4/glBlendColor.xhtml")?.chars().collect::<Vec<_>>();
    println!("{:?}", example);
    
    let lex = Lexer::new(&example);
    // Here you would use the lexer to process the content
    Ok(())

}