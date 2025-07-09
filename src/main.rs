use std::fs::{self, File};
use std::io::{self, Read};
use std::process;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;

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
        if self.content[0].is_alphabetic() {
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
    
    for token in Lexer::new(&example) {
        println!("{:?}", token);
    }
    // Here you would use the lexer to process the content
    Ok(())

}