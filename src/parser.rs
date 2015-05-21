use std::collections::HashMap;

pub type ParseResult<T> = Result<T, ParseError>;

pub struct ParseError {
  pub pos: u8,
}

pub enum Value {
  Str(String),
  Hash(HashMap<String, Value>, Option<String>)
}

pub enum Entry {
  Entity {id: String, value: Value}
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
        None => break
      };

      if ch == '<' {
        entries.push(self.parse_entry());
      }
    }
    entries
  }

  fn parse_entry(&mut self) -> Entry {
    let id = self.parse_identifier();
    let val = match self.ch {
      Some(_) => self.parse_entity(id),
      None => panic!()
    };

    self.parse_whitespace();
    if !self.ch_is('>') {
      panic!();
    }
    self.bump();
    val
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

  fn parse_value(&mut self) -> Value {
    match self.ch {
      Some('"') | Some('\'') => self.parse_string(),
      Some('{') => self.parse_hash(),
      _ => panic!()
    }
  }

  fn parse_string(&mut self) -> Value {
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
    Value::Str(s)
  }

  fn parse_hash(&mut self) -> Value {
    self.bump();
    self.parse_whitespace();

    let mut map = HashMap::new();
    let mut default = None;

    loop {
      let id = self.parse_identifier();
      self.parse_whitespace();
      self.bump();
      self.parse_whitespace();
      let value = self.parse_value();
      self.bump();
      map.insert(id, value);
      break;
    }
    Value::Hash(map, default)
  }
}

fn read_file() -> String {
  let s = "<entity1 {one: 'foo'}> <entity2 'foo'>".to_string();
  return s
}

fn main() {
  let source = read_file();
  let mut parser = Parser::new(source.trim());
  let mut entries = parser.parse();

  loop {
    if entries.is_empty() {
      break;
    }
    let entry1 = Some(entries.remove(0));
    let id = match entry1 {
      Some(Entry::Entity{id, value}) => id.clone(),
      None => break
    };

    println!("ID: {}", id);
  }
}
