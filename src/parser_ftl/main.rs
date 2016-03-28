pub enum Expression {
  IdentifierExpression {name: String}
}

pub enum PatternElement {
  TextElement {value: String},
  Placeable {expressions: Vec<Expression>}
}

pub enum Value {
  Pattern {source: String, elements: Vec<PatternElement>}
}

pub enum Entry {
  Entity {id: Identifier, value: Value}
}

pub struct Identifier {
  pub name: String,
  pub namespace: String
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

  fn get_ws(&mut self) {
    while self.ch_is(' ') ||
          self.ch_is('\n') ||
          self.ch_is('\t') ||
          self.ch_is('\r') { self.bump(); }
  }

  fn get_line_ws(&mut self) {
    while self.ch_is(' ') ||
          self.ch_is('\t') { self.bump(); }
  }

  pub fn parse(&mut self) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

    self.get_ws();

    self.bump();

    loop {
      if self.ch == None {
        break;
      }

      entries.push(self.get_entry());
      self.get_ws();
    }
    entries
  }

  fn get_entry(&mut self) -> Entry {
    let val = self.get_entity();

    val
  }

  fn get_entity(&mut self) -> Entry {
    let id = self.get_identifier(true);
    self.get_line_ws();

    if !self.ch_is('=') {
      panic!();
    }
    self.bump();

    self.get_line_ws();

    let value = self.get_pattern();

    Entry::Entity{id: id, value: value}
  }

  fn get_identifier(&mut self, ns_sep: bool) -> Identifier {
    let mut id = String::new();
    let mut namespace = String::new();

    if ns_sep {
      let ns = self.get_identifier(false);
      if self.ch_is(':') {
        namespace = ns.name;
        self.bump();
      } else if ns.name.len() > 0 {
        id = ns.name;
      }
    }

    let ch = match self.ch {
      Some(c) => c,
      None => panic!(),
    };

    match ch {
      'a'...'z' | 'A'...'Z' | '_' => id.push(ch),
        _ => return Identifier{name: id, namespace: namespace},
    }
    self.bump();

    loop {
      let ch = match self.ch {
        Some(c) => c,
        None => break,
      };

      match ch {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '-' => id.push(ch),
        _ => break,
      }
      self.bump();
    }

    Identifier{name: id, namespace: namespace}
  }

  fn get_pattern(&mut self) -> Value {
    let mut s = String::new();
    let mut elements = vec![];

    loop {
      match self.ch {
        Some(c) if c == '\n' => { self.bump(); break },
        Some(c) => s.push(c),
        None => { break }
      }
      self.bump();
    }

    elements.push(PatternElement::TextElement {value: s.clone()});

    Value::Pattern{source: s, elements: elements}
  }
}
