#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate getopts;

use self::ftl::ast::parser::Parser as FTLParser;
use self::ftl::entries::parser::Parser as EntriesParser;
use self::ftl::ast::ast::Resource as ASTResource;
use self::ftl::entries::ast::Resource as EntriesResource;

use std::fs::File;
use std::io::Read;
use std::env;
use std::io;

use getopts::Options;

fn read_file(path: &String) -> Result<String, io::Error> {
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

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("s", "silence", "disable output");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };


    let source = read_file(&input).expect("Read file failed");
    let res = if input.contains(".json") {
        deserialize_json(source.trim())
    } else {
        let mut parser = EntriesParser::new(source.trim());
        parser.parse()
    };

    if !matches.opt_present("s") {
        print_entries_resource(&res);
    }
}

pub mod ftl;
