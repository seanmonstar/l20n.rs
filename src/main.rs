extern crate parser_l20n;
extern crate parser_ftl;

use parser_l20n::Parser as L20nParser;
use parser_l20n::Entry as L20nEntry;

use parser_ftl::Parser as FTLParser;
use parser_ftl::Entry as FTLEntry;

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
  reader.read_to_string(buffer_string)
    .ok()
    .expect("Failed to read string");
  
  buffer_string.clone()
}

fn print_l20n_entities(entries: &mut Vec<L20nEntry>) {
  loop {
    if entries.is_empty() {
      break;
    }
    let entry1 = Some(entries.remove(0));
    let id = match entry1 {
      Some(L20nEntry::Entity{ id, .. }) => id.clone(),
      None => break
    };

    println!("ID: {}", id);
  }
}

fn print_ftl_entities(entries: &mut Vec<FTLEntry>) {
  loop {
    if entries.is_empty() {
      break;
    }
    let entry1 = Some(entries.remove(0));
    let id = match entry1 {
      Some(FTLEntry::Entity{ id, .. }) => id.clone(),
      None => break
    };

    println!("ID: {}", id);
  }
}

fn main() {
  if let Some(arg1) = env::args().nth(1) {
    let source = read_file(arg1.clone());
    if arg1.ends_with(".ftl") {
      let mut parser = FTLParser::new(source.trim());
      let mut entries = parser.parse();
      print_ftl_entities(&mut entries);
    } else {
      let mut parser = L20nParser::new(source.trim());
      let mut entries = parser.parse();
      print_l20n_entities(&mut entries);
    }
  } else {
    println!("You must pass a path to an l20n file");
    return;
  }
}