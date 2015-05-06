
pub struct ParseError {
  /// The line where the error occurred.
  pub line: u8,
}


pub struct Parser {
  pos: u8,
  source: String
}

pub enum Entry {
  Entity(String)
}

pub enum Value {
  Str(String)
}

impl Parser {
  pub fn new(source: String) -> Parser {
    Parser {
      pos: 0,
      source: source
    }
  }

  pub fn parse(&self) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    loop {
      let ch = match self.source.chars().next() {
        Some(ch) => ch,
        None => { break; }
      };

      if ch == '<' {
        entries.push(self.parse_entry());
      }
    }
    entries
  }

  pub fn parse_entry(&self) -> Entry {
    let id = self.parse_identifier();
    self.parse_entity(id)
  }

  pub fn parse_entity(&self, id: String) -> Entry {
    Entry::Entity(id)
  }

  pub fn parse_identifier(&self) -> String {
    let mut id = String::new();
    id.push('a');
    id
  }
}

/*  ---------------------- */

fn read_file() -> String {
  let s = "<entity1 'Value'>".to_string();
  return s
}

fn main() {
  let source = read_file();
  let parser = Parser::new(source);
  let entries = parser.parse();

  let entry1 = entries.pop();

  let id = match entry1 {
    Entry::Entity(ref id) => id.clone(),
  };

  println!("The result is {}", id);
}
