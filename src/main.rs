use self::ftl::parser::Parser as FTLParser;
use self::ftl::ast::Entry as FTLEntry;
use self::ftl::ast::Value as FTLValue;
use self::ftl::ast::Identifier as FTLIdentifier;
use self::ftl::ast::Keyword as FTLKeyword;
use self::ftl::ast::Member as FTLMember;

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


fn get_ftl_id(id: &FTLIdentifier) -> String {
    return id.name.to_string();
}

fn get_ftl_key(key: &FTLKeyword) -> String {
    return key.name.to_string();
}

fn print_ftl_entities(entries: &mut Vec<FTLEntry>) {
    loop {
        if entries.is_empty() {
            break;
        }
        match entries.remove(0) {
            FTLEntry::Entity { id, value, traits } => {
                let FTLValue::Pattern { source, .. } = value;
                let traits: Option<Vec<FTLMember>> = traits;

                println!("ID: {}, VALUE: {}", get_ftl_id(&id), source);
                match traits {
                    Some(t) => print_ftl_traits(t),
                    None => {}
                }
            }
            FTLEntry::Comment { content } => {
                println!("Comment: {}", content);
            }
            FTLEntry::Section { key, .. } => {
                println!("Section: {}", get_ftl_key(&key));
            }
        }
    }
}

fn print_ftl_traits(traits: Vec<FTLMember>) {
    for t in &traits {
        let FTLValue::Pattern { ref source, .. } = t.value;
        println!("  Trait: {}, Value: {}", get_ftl_key(&t.key), source);

    }
}

fn main() {
    if let Some(arg1) = env::args().nth(1) {
        let source = read_file(arg1.clone()).expect("Read file failed");
        let mut parser = FTLParser::new(source.trim());
        let mut entries = parser.parse();
        print_ftl_entities(&mut entries);
    } else {
        println!("You must pass a path to an l20n file");
        return;
    }
}

pub mod ftl;
