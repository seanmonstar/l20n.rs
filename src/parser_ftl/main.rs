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
  pub name: String
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
    let id = self.get_identifier();
    self.get_line_ws();

    if !self.ch_is('=') {
      panic!();
    }
    self.bump();

    self.get_line_ws();

    let value = self.get_pattern();

    Entry::Entity{id: id, value: value}
  }

  fn get_identifier(&mut self) -> Identifier {
    let mut name = String::new();

    let ch = match self.ch {
      Some(c) => c,
      None => panic!(),
    };

    match ch {
      'a'...'z' | 'A'...'Z' | '_' => name.push(ch),
        _ => return Identifier{name: name},
    }
    self.bump();

    loop {
      let ch = match self.ch {
        Some(c) => c,
        None => break,
      };

      match ch {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '-' => name.push(ch),
        _ => break,
      }
      self.bump();
    }

    Identifier{name: name}
  }

  fn get_pattern(&mut self) -> Value {
    let mut buffer = String::new();
    let mut source = String::new();
    let mut content = vec![];
    let mut quote_delimited: bool = false;
    let mut first_line = true;

    if self.ch_is('"') {
      quote_delimited = true;
    }

    loop {
      match self.ch {
        Some(c) if c == '\n' => {
          if quote_delimited {
            panic!("Unclosed string");
          }
          self.bump();
          self.get_line_ws();

          if !self.ch_is('|') {
            break;
          }
          if first_line && buffer.len() != 0 {
            panic!("Multiline string should have the ID line empty");
          }
          first_line = false;
          self.bump();
          if self.ch_is(' ') {
            self.bump();
          }
          if buffer.len() != 0 {
            buffer.push('\n');
          }
          continue;
        },
        Some(c) if c == '"' => {
          self.bump();
          quote_delimited = false;
          break;
        },
        Some(c) => source.push(c),
        None => { break }
      }
      match self.ch {
        Some(c) => buffer.push(c),
        None => continue,
      };
      self.bump();
    }

    if quote_delimited {
      panic!("Unclosed string");
    }

    if buffer.len() != 0 {
      //source.append(buffer);
      content.push(PatternElement::TextElement {value: source.clone()});
    }

    if content.len() == 0 {
      //return Value::Pattern(source: source, elements: content);
    }

    content.push(PatternElement::TextElement {value: source.clone()});

    Value::Pattern{source: source, elements: content}
  }
}
