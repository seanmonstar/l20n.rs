extern crate parser;

use parser::Parser;
use parser::Entry;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use std::env;
 
fn read_file(path: String) -> String {
  let file = match File::open(path) {
      Ok(file) => file,
      Err(..)  => panic!("room"),
  };

  let mut reader = BufReader::new(&file);
  let buffer_string = &mut String::new();
  reader.read_to_string(buffer_string);
  
  buffer_string.clone()
}

fn print_entities(entries: &mut Vec<Entry>) {
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

fn main() {
  if let Some(arg1) = env::args().nth(1) {
    let source = read_file(arg1);
    let mut parser = Parser::new(source.trim());
    let mut entries = parser.parse();
    print_entities(&mut entries);
  } else {
    println!("You must pass a path to an l20n file");
    return;
  }
}
