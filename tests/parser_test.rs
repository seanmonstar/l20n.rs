extern crate l20n;
extern crate rustc_serialize;

use std::io::prelude::*;
use std::fs::File;
use std::io;
use rustc_serialize::json::{Json, ToJson};
use std::collections::BTreeMap;

use l20n::ftl::parser::Parser as FTLParser;
use l20n::ftl::ast::Entry as FTLEntry;
use l20n::ftl::ast::Value as FTLValue;

fn read_file(path: &str) -> Result<String, io::Error> {
  let mut f = try!(File::open(path));
  let mut s = String::new();
  try!(f.read_to_string(&mut s));
  Ok(s)
}

fn read_json_file(path: &str) -> Result<Json, io::Error> {
  let mut f = try!(File::open(path));
  let mut data = String::new();
  try!(f.read_to_string(&mut data));

  let json = Json::from_str(&data).unwrap();
  Ok(json)
}

fn resource_to_json(entries: &Vec<FTLEntry>) -> Json {
  let mut d = BTreeMap::new();

  for i in 0..entries.len() {
    if let FTLEntry::Entity{ ref id, ref value, ref traits } = entries[i] {
      let FTLValue::Pattern { ref source, .. } = *value;
      d.insert(id.name.to_string(), source.to_json());
    }
  }

  Json::Object(d)
}

#[test]
fn it_works() {
  let string = read_file("./tests/fixtures/parser/ftl/01-basic01.ftl").expect("Failed to read");

  let reference_json = read_json_file("./tests/fixtures/parser/ftl/01-basic01.entries.json").expect("Failed to read");

  let mut parser = FTLParser::new(string.trim());

  let mut entries = parser.parse();

  let json = resource_to_json(&entries);

  assert_eq!(json, reference_json);
}
