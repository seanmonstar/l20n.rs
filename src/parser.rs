pub enum Entry {
  Entity {id: String, value: String}
}

pub enum Value {
  Str(String)
}

pub struct Parser<'a> {
  source: std::str::Chars<'a>,
  ch: Option<char>,
  pos: u8
}

impl<'a> Parser<'a> {
  pub fn new(source: &'a str) -> Parser<'a> {
    Parser {
      source: source.chars(),
      ch: None,
      pos: 0
    }
  }

  fn bump(&mut self) {
    self.ch = self.source.next();

    self.pos += 1;
  }

  fn ch_is(&self, ch: char) -> bool {
    self.ch == Some(ch)
  }

  fn parse_whitespace(&mut self) {
    while self.ch_is(' ') ||
          self.ch_is('\n') ||
          self.ch_is('\t') ||
          self.ch_is('\r') { self.bump(); }
  }

  pub fn parse(&mut self) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    self.bump();

    loop {
      self.parse_whitespace();
      let ch = match self.ch {
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

  fn parse_entry(&mut self) -> Entry {
    let id = self.parse_identifier();
    match self.ch {
      Some(_) => return self.parse_entity(id),
      None => panic!()
    };
  }

  fn parse_entity(&mut self, id: String) -> Entry {
    if !self.ch_is(' ') {
      panic!();
    }
    self.parse_whitespace();
    let value = self.parse_value();
    Entry::Entity{id: id, value:value}
  }

  fn parse_identifier(&mut self) -> String {
    let mut id = String::new();

    loop {
      self.bump();
      let ch = match self.ch {
        Some(c) => c,
        None => break,
      };

      match ch {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => id.push(ch),
        _ => break,
      }
    }

    id
  }

  fn parse_value(&mut self) -> String {
    match self.ch {
      Some('"') | Some('\'') => self.parse_string(),
      _ => panic!()
    }
  }

  fn parse_string(&mut self) -> String {
    let mut s = String::new();
    let quote = self.ch.unwrap();

    loop {
      self.bump();
      match self.ch {
        Some(c) if c == quote => { self.bump(); break },
        Some(c) => s.push(c),
        None => panic!()
      }
    }
    s
  }
}

fn main() {
  //let v: std::str::Chars<'a> = "abc åäö".chars();
  let mut parser = Parser::new("<entity1 \"foo\">");
  let mut entries = parser.parse();

  let entry1 = entries.pop();
  let (id, value) = match entry1 {
    Some(Entry::Entity{id, value}) => (id.clone(), value.clone()),
    None => ("".to_string(), "".to_string())
  };

  println!("The result id is {}", id);
  println!("The result value is {}", value);

}
