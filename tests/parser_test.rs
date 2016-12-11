extern crate l20n;
extern crate serde;
extern crate serde_json;

use std::io::prelude::*;
use std::fs::File;
use std::io;
use std::fs;

use l20n::ftl::entries::parser::Parser as EntriesParser;
use l20n::ftl::entries::ast::Resource as EntriesResource;

fn read_file(path: &str) -> Result<String, io::Error> {
  let mut f = try!(File::open(path));
  let mut s = String::new();
  try!(f.read_to_string(&mut s));
  Ok(s)
}

fn read_json_file(path: &str) -> Result<EntriesResource, io::Error> {
  let mut f = try!(File::open(path));
  let mut data = String::new();
  try!(f.read_to_string(&mut data));

  let json = serde_json::from_str(&data).unwrap();
  Ok(json)
}

#[test]
fn entries_parser() {
  let paths = fs::read_dir("./tests/fixtures/parser/ftl/").unwrap();

  for p in paths {
      let path = p.unwrap().path();
      if path.extension().unwrap().to_str().unwrap() != "ftl" ||
         path.to_str().unwrap().contains("errors") {
        continue;
      }

      let path_len = path.to_str().unwrap().len();
      let entries_path = format!("{}.entries.json", &path.to_str().unwrap()[0..(path_len-4)]);
      let string = read_file(path.to_str().unwrap()).expect("Failed to read");
      let reference_res = read_json_file(&entries_path).expect("Failed to read");

      let mut parser = EntriesParser::new(string.trim());

      let res = parser.parse();

      println!("Comparing file: {}", path.to_str().unwrap());

      assert_eq!(reference_res, res, "Foo");
  }
}
