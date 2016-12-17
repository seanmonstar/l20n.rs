extern crate serde_json;
extern crate core;

use std::str;
use self::core::iter::Peekable;
use super::ast::*;
use self::serde_json::Map;


pub struct Parser<'a> {
    source: Peekable<str::Chars<'a>>,
    ch: Option<char>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        Parser {
            source: source.chars().peekable(),
            ch: None,
        }
    }

    fn bump(&mut self) {
        self.ch = self.source.next();
    }

    fn ch_is(&self, ch: char) -> bool {
        self.ch == Some(ch)
    }

    fn get_ws(&mut self) {
        while self.ch_is(' ') || self.ch_is('\n') || self.ch_is('\t') || self.ch_is('\r') {
            self.bump();
        }
    }

    fn get_line_ws(&mut self) {
        while self.ch_is(' ') || self.ch_is('\t') {
            self.bump();
        }
    }

    pub fn parse(&mut self) -> Resource {
        let mut entries = Map::new();

        self.get_ws();

        self.bump();

        loop {
            if self.ch == None {
                break;
            }

            self.get_entry(&mut entries);
            self.get_ws();
        }
        Resource(entries)
    }

    fn get_entry(&mut self, entries: &mut Map<String, Value>) {
        if self.ch_is('#') {
            self.get_comment();
            return;
        }

        if self.ch_is('[') {
            self.get_section();
            return;
        }

        if self.ch != None && !self.ch_is('\n') {
            self.get_entity(entries);
        }
    }

    fn get_section(&mut self) {
        self.bump();
        self.bump();

        self.get_line_ws();

        self.get_keyword();

        self.get_line_ws();

        self.bump();
        self.bump();
    }


    fn get_entity(&mut self, entries: &mut Map<String, Value>) {
        let id = self.get_identifier();
        self.get_line_ws();

        if !self.ch_is('=') {
            panic!();
        }
        self.bump();

        self.get_line_ws();

        let value = self.get_pattern();

        if self.ch_is('\n') {
            self.bump();
            self.get_line_ws();
        }

        if (self.ch_is('[') && self.source.peek() != Some(&'[')) ||
           self.ch_is('*') {
            let (members, default_index) = self.get_members();

            entries.insert(id, Value::ComplexValue{
              val: match value {
                Pattern::Simple(v) => {
                  if v.is_empty() {
                    None
                  } else {
                    Some(Pattern::Simple(v))
                  }
                },
                Pattern::Complex(v) => {
                  Some(Pattern::Complex(v))
                }
              },
              traits: Some(members),
              def: default_index
            });
        } else {
            entries.insert(id, Value::Pattern(value));
        }

    }

    fn get_members(&mut self) -> (Vec<Member>, Option<i8>) {
        let mut members = vec![];
        let mut default_index: Option<i8> = None;
        let mut index = 0;

        loop {
            if (!self.ch_is('[') || self.source.peek() == Some(&'[')) &&
               !self.ch_is('*') {
                break;
            }

            let def = self.ch_is('*');

            if def {
              self.bump();
              default_index = Some(index);
            }

            self.bump();

            let keyword = self.get_keyword();

            self.bump();

            self.get_line_ws();

            let member = Member {
              key: keyword,
              val: self.get_pattern(),
            };
            members.push(member);
            index += 1;

            self.get_ws()

        }

        (members, default_index)
    }

    fn get_comment(&mut self) {
        self.bump();
        if self.ch_is(' ') {
            self.bump();
        }

        loop {
            while !self.ch_is('\n') {
                if self.ch == None {
                    break;
                }

                self.bump();
            }

            self.bump();

            if self.ch_is('#') {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn get_identifier(&mut self) -> String {
        let mut name = String::new();

        let ch = match self.ch {
            Some(c) => c,
            None => panic!(),
        };

        match ch {
            'a'...'z' | 'A'...'Z' | '_' => name.push(ch),
            _ => return name,
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

        name
    }

    fn get_keyword(&mut self) -> Keyword {
        let mut name = String::new();
        let mut namespace = self.get_identifier();

        if self.ch_is('/') {
          self.bump();
        } else {
          name = namespace;
          namespace = String::new();
        }

        let ch = match self.ch {
            Some(c) => c,
            None => panic!(),
        };

        match ch {
            'a'...'z' | 'A'...'Z' | '_' => {
              self.bump();
              name.push(ch)
            },
            _ => if name.is_empty() { panic!() },
        }

        loop {
            let ch = match self.ch {
                Some(c) => c,
                None => break,
            };

            match ch {
                'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '-' | ' ' | '/' => name.push(ch),
                _ => break,
            }
            self.bump();
        }

        Keyword {
          t: String::from("kw"),
          name: name,
          ns: if namespace.is_empty() { None } else { Some(namespace) }
        }
    }

    fn get_pattern(&mut self) -> Pattern {
        let mut buffer = String::new();
        let mut content: Vec<PatternElement> = vec![];
        let mut quote_delimited: bool = false;
        let mut first_line = true;

        if self.ch_is('"') {
            quote_delimited = true;
            self.bump();
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
                    
                    if !first_line {
                        buffer.push('\n');
                    }
                    first_line = false;
                    self.bump();
                    if self.ch_is(' ') {
                        self.bump();
                    }
                    continue;
                }
                Some(c) if c == '\\' => {
                    self.bump();
                    if let Some(ch2) = self.ch {
                      if (quote_delimited && ch2 == '"') || ch2 =='{' {
                          buffer.push(ch2);
                          self.bump();
                          continue;
                      }
                    }
                }
                Some(c) if c == '"' => {
                    self.bump();
                    quote_delimited = false;
                    break;
                }
                Some(c) if c == '{' => {
                  
                }
                Some(c) => buffer.push(c),
                None => break,
            }
            self.bump();
        }

        if quote_delimited {
            panic!("Unclosed string");
        }

        Pattern::Simple(buffer)
    }

}
