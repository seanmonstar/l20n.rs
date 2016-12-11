extern crate serde_json;

use std::str;
use super::ast::*;
use self::serde_json::Map;


pub struct Parser<'a> {
    source: str::Chars<'a>,
    ch: Option<char>,
    pos: u32,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser<'a> {
        Parser {
            source: source.chars(),
            ch: None,
            pos: 0,
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

            self.get_entity(&mut entries);
            self.get_ws();
        }
        Resource(entries)
    }

    pub fn get_entity(&mut self, entries: &mut Map<String, String>) {
        let id = self.get_identifier();
        self.get_line_ws();

        if !self.ch_is('=') {
            panic!();
        }
        self.bump();

        self.get_line_ws();

        let value = self.get_pattern();
        entries.insert(id, value);
    }

    pub fn get_identifier(&mut self) -> String {
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

    fn get_pattern(&mut self) -> String {
        let mut source = String::new();
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
                    if first_line && source.len() != 0 {
                        panic!("Multiline string should have the ID line empty");
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
                          source.push(ch2);
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
                Some(c) => source.push(c),
                None => break,
            }
            self.bump();
        }

        if quote_delimited {
            panic!("Unclosed string");
        }

        if source.len() == 0 {
            return source;
        }

        source
    }

}
