use std::collections::HashMap;

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
  pos: u16
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

    self.parse_whitespace();

    self.bump();

    loop {
      if self.ch == None {
        break;
      }

      entries.push(self.parse_entry());
      self.parse_whitespace();
    }
    entries
  }

  fn parse_entry(&mut self) -> Entry {
    let val = self.parse_entity();

    val
  }

  fn parse_entity(&mut self) -> Entry {
    let id = self.parse_identifier();
    self.parse_whitespace();

    if !self.ch_is('=') {
      panic!();
    }
    self.bump();

    self.parse_whitespace();

    let value = self.parse_pattern();

    Entry::Entity{id: id, value:value}
  }

  fn parse_identifier(&mut self) -> String {
    let mut id = String::new();

    loop {
      let ch = match self.ch {
        Some(c) => c,
        None => break,
      };

      match ch {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => id.push(ch),
        _ => break,
      }
      self.bump();
    }

    id
  }

  fn parse_pattern(&mut self) -> Value {
    let mut s = String::new();

    loop {
      self.bump();
      match self.ch {
        Some(c) if c == '\n' => { self.bump(); break },
        Some(c) => s.push(c),
        None => { break }
      }
    }
    Value::Str(s)
  }
}
