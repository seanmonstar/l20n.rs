#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use self::ftl::ast::parser::Parser as FTLParser;
use self::ftl::entries::parser::Parser as EntriesParser;
use self::ftl::ast::ast::Resource as ASTResource;
use self::ftl::entries::ast::Resource as EntriesResource;

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

fn print_ast_resource(res: &ASTResource) {
    let e = serde_json::to_string_pretty(res).unwrap();
    println!("{}", e);
}

fn print_entries_resource(res: &EntriesResource) {
    let e = serde_json::to_string_pretty(res).unwrap();
    println!("{}", e);
}

fn deserialize_json(source: &str) -> EntriesResource {
    return serde_json::from_str(source).unwrap();
}

fn main() {
    if let Some(arg1) = env::args().nth(1) {
        let source = read_file(arg1.clone()).expect("Read file failed");
        let res = if arg1.contains(".json") {
          deserialize_json(source.trim())
        } else {
          let mut parser = EntriesParser::new(source.trim());
          parser.parse()
        };
        print_entries_resource(&res);
    } else {
        println!("You must pass a path to an l20n file");
        return;
    }
}

pub mod ftl;
