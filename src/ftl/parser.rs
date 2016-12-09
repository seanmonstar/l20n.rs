use std::str;
use super::ast::*;


pub struct Parser<'a> {
    source: str::Chars<'a>,
    ch: Option<char>,
    pos: u16,
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

    pub fn parse(&mut self) -> Vec<Entry> {
        let mut entries: Vec<Entry> = Vec::new();

        self.get_ws();

        self.bump();

        loop {
            if self.ch == None {
                break;
            }

            let entry = self.get_entry();
            if let Some(x) = entry {
                entries.push(x);
            }
            self.get_ws();
        }
        entries
    }

    fn get_entry(&mut self) -> Option<Entry> {
        let mut comment: Option<Entry> = None;

        if self.ch_is('#') {
            comment = Some(self.get_comment());
        }

        self.get_line_ws();

        if self.ch_is('[') {
            return Some(self.get_section());
        }

        if self.ch != None && !self.ch_is('\n') {
            return Some(self.get_entity());
        }

        comment
    }

    fn get_section(&mut self) -> Entry {
        self.bump();
        self.bump();

        self.get_line_ws();

        let key = self.get_keyword();

        self.get_line_ws();

        self.bump();
        self.bump();

        let mut body = vec![];
        Entry::Section(Section {
            key: key,
            body: body,
        })
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

        if self.ch_is('\n') {
            self.bump();
            self.get_line_ws();
        }

        let mut members: Option<Vec<Member>> = None;
        if self.ch_is('[') {
            members = Some(self.get_members());
        }

        Entry::Entity(Entity {
            id: id,
            value: value,
            traits: members,
        })
    }

    fn get_identifier(&mut self) -> Identifier {
        let mut name = String::new();

        let ch = match self.ch {
            Some(c) => c,
            None => panic!(),
        };

        match ch {
            'a'...'z' | 'A'...'Z' | '_' => name.push(ch),
            _ => return Identifier(name),
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

        Identifier(name)
    }

    fn get_keyword(&mut self) -> Keyword {
        let mut name = String::new();

        let ch = match self.ch {
            Some(c) => c,
            None => panic!(),
        };

        match ch {
            'a'...'z' | 'A'...'Z' | '_' => name.push(ch),
            _ => return Keyword(name),
        }
        self.bump();

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

        Keyword(name)
    }

    fn get_pattern(&mut self) -> Option<Pattern> {
        let mut source = String::new();
        let mut content = vec![];
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
            return None;
        }

        content.push(PatternElement::TextElement(TextElement(source.clone())));

        Some(Pattern {
            source: source,
            elements: content,
        })
    }

    fn get_members(&mut self) -> Vec<Member> {
        let mut members = vec![];

        loop {
            if !self.ch_is('[') {
                break;
            }

            self.bump();

            let key = self.get_keyword();

            self.bump();

            self.get_line_ws();

            let member = Member {
              key: key,
              value: self.get_pattern(),
              default: false,
            };
            members.push(member);

            self.get_ws()

        }

        members
    }

    fn get_comment(&mut self) -> Entry {
        self.bump();
        if self.ch_is(' ') {
            self.bump();
        }

        let mut content = String::new();

        loop {
            while !self.ch_is('\n') {
                let ch = match self.ch {
                    Some(c) => c,
                    None => break,
                };

                content.push(ch);
                self.bump();
            }

            self.bump();

            if self.ch_is('#') {
                content.push('\n');
                self.bump();
            } else {
                break;
            }
        }

        Entry::Comment(Comment ( content ))

    }
}
