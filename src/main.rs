use std::fs;
use std::process;
fn main() {
    let file_path = "docs.gl/gl4/glClear.xhtml";
    let content = fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("ERROR: could not read file {file_path}: {err}");
        process::exit(1);
    });
    println!("length of content in {file_path} is {length}", length = content.len()); 
}
