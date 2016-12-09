#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use self::ftl::parser::Parser as FTLParser;
use self::ftl::ast::*;

use std::fs::File;
use std::io::Read;
use std::env;
use std::io;

fn read_file(path: String) -> Result<String, io::Error> {
  let mut f = try!(File::open(path));
  let mut s = String::new();
  try!(f.read_to_string(&mut s));
  Ok(s)
}

fn print_ftl_json(entries: &Vec<Entry>) {
    for entry in entries {
        match entry {
            &Entry::Entity(ref entity) => {
              let e = serde_json::to_string(&entity).unwrap();
              println!("Entity: {}", e);
            }
            &Entry::Comment(ref comment) => {
              println!("Comment: {}", serde_json::to_string(&comment).unwrap());
            }
            &Entry::Section(Section { .. }) => {
            }
        }
    }
}

fn main() {
    if let Some(arg1) = env::args().nth(1) {
        let source = read_file(arg1.clone()).expect("Read file failed");
        let mut parser = FTLParser::new(source.trim());
        let entries = parser.parse();
        //print_ftl_entities(&entries);
        print_ftl_json(&entries);
    } else {
        println!("You must pass a path to an l20n file");
        return;
    }
}

pub mod ftl;
