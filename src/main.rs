use std::fs::{self, File};
use std::io::{self, Read};
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

fn main() -> io::Result<()>{
    let dir_path = "docs.gl/gl4";
    if !fs::metadata(dir_path).is_ok() {
        eprintln!("Directory {} does not exist or is not accessible.", dir_path);
        process::exit(1);
    }
    let dir = fs::read_dir(dir_path).unwrap_or_else(|err| {
        eprintln!("Error reading directory {}: {}", dir_path, err);
        process::exit(1);
    });

    for file in dir {
        let file_path = file?.path();
        let content = read_entire_xml_file(file_path.to_str().unwrap()).unwrap_or_else(|err| {
            eprintln!("Error reading file {:?}: {}", file_path, err);
            process::exit(1);
        });
        println!("{}", content);
    }
    Ok(())
}