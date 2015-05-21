extern crate parser;

use parser::Parser;
use parser::Entry;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;
 
fn read_file() -> String {
  let file = match File::open("example.l20n") {
      Ok(file) => file,
      Err(..)  => panic!("room"),
  };

  let mut reader = BufReader::new(&file);
  let buffer_string = &mut String::new();
  reader.read_to_string(buffer_string);
  
  buffer_string.clone()
}

fn main() {
  let source = read_file();
  let mut parser = Parser::new(source.trim());
  let mut entries = parser.parse();

  loop {
    if entries.is_empty() {
      break;
    }
    let entry1 = Some(entries.remove(0));
    let id = match entry1 {
      Some(Entry::Entity{id, value}) => id.clone(),
      None => break
    };

    println!("ID: {}", id);
  }
}
