pub struct ParseError {
  pub line: u8,
}


pub struct Parser {
  pos: u8,
  source: String,
  ch: Option<char>
}

pub enum Entry {
  Entity {id: String}
}

pub enum Value {
  Str(String)
}

impl Parser {
  pub fn new(source: String) -> Parser {
    Parser {
      pos: 0,
      source: source,
      ch: None
    }
  }

  fn bump(&mut self) {
    self.ch = self.source.chars().next();

    self.pos += 1;
  }

  pub fn parse(&mut self) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    loop {
      let ch = match self.source.chars().next() {
        Some(ch) => ch,
        None => { break; }
      };

      if ch == '<' {
        entries.push(self.parse_entry());
        break;
      }
    }
    entries
  }

  pub fn parse_entry(&mut self) -> Entry {
    self.bump();
    let id = self.parse_identifier();
    self.parse_entity(id)
  }

  pub fn parse_entity(&self, id: String) -> Entry {
    Entry::Entity{id: id}
  }

  pub fn parse_identifier(&mut self) -> String {
    let mut id = String::new();

    loop {
      self.bump();
      let ch = match self.ch {
        Some(c) => c,
        None => break,
      };

      id.push(ch);
      break;
    }

    id
  }
}

/*  ---------------------- */

fn read_file() -> String {
  let s = "<entity1>".to_string();
  return s
}

fn main() {
  let source = read_file();
  let mut parser = Parser::new(source);
  let mut entries = parser.parse();

  let entry1 = entries.pop();

  let id = match entry1 {
    Some(Entry::Entity{id}) => id.clone(),
    None => "".to_string()
  };

  println!("The result is {}", id);
}
