use std::io::{self, Read};
use std::fs::File;
use std::process;
use xml::reader::{EventReader, XmlEvent};

fn read_entire_xml_file(file_path: &str) -> io::Result<String> {
    let file = File::open(file_path)?;
    let er = EventReader::new(file);

    let mut content = String::new(); // buffer to hold the content

    for event in er.into_iter() {
        match event {
            Ok(XmlEvent::Characters(text)) => {
                content.push_str(&text); // append text to the content
            }
            Err(e) => {
                eprintln!("Error reading XML event: {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, "XML parsing error"));
            }
            _ => {} // ignore other events
        }
    }

    Ok(content)
}

fn main() {
    let file_path = "docs.gl/gl4/glClear.xhtml";
    let content = read_entire_xml_file(file_path).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", file_path, err);
        process::exit(1);
    });
    println!("{}", content);
}
