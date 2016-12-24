extern crate serde_json;
extern crate core;
extern crate itertools;

use std::str;
use super::ast::*;
use self::serde_json::Map;
use self::itertools::MultiPeek;
use self::itertools::multipeek;

fn is_id_start(ch: char) -> bool {
    match ch {
        'a'...'z' | 'A'...'Z' | '_' => true,
        _ => false,
    }
}

fn is_id_char(ch: char) -> bool {
    match ch {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '-' => true,
        _ => false,
    }
}

fn is_kw_char(ch: char) -> bool {
    match ch {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '-' | ' ' | '/' => true,
        _ => false,
    }
}


pub struct Parser<'a> {
    source: MultiPeek<str::Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &str) -> Parser {
        Parser { source: multipeek(source.chars()) }
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.source.peek()
    }

    fn peek_char_eq(&mut self, ch: char) -> bool {
        match self.peek_char() {
            Some(&peek_ch) => peek_ch == ch,
            None => false,
        }
    }

    fn first_non_ws_char_eq(&mut self, ch2: char) -> bool {
        while let Some(&ch) = self.peek_char() {
            if ch != ' ' && ch != '\n' && ch != '\t' && ch != '\r' {
                if ch == ch2 {
                    return true;
                }
                break;
            }

            self.bump();
        }

        return false;
    }

    fn read_char(&mut self) -> Option<char> {
        self.source.next()
    }

    fn bump(&mut self) {
        self.source.next();
    }

    fn skip_ws(&mut self) {
        while let Some(&ch) = self.peek_char() {
            if ch != ' ' && ch != '\n' && ch != '\t' && ch != '\r' {
                self.source.reset_peek();
                break;
            }

            self.bump();
        }
    }

    fn skip_line_ws(&mut self, reset: bool) {
        while let Some(&ch) = self.peek_char() {
            if ch != ' ' && ch != '\t' {
                if reset {
                    self.source.reset_peek();
                }
                break;
            }

            self.bump();
        }
    }

    pub fn parse(&mut self) -> Resource {
        let mut entries = Map::new();

        self.skip_ws();

        loop {
            match self.peek_char() {
                Some(_) => {
                    self.source.reset_peek();
                    self.get_entry(&mut entries);
                    self.skip_ws();
                }
                None => {
                    break;
                }
            }
        }
        Resource(entries)
    }

    fn get_entry(&mut self, entries: &mut Map<String, Value>) {
        match self.peek_char() {
            Some(&ch) => {
                match ch {
                    '#' => {
                      return self.get_comment();
                    }
                    '[' => {
                      return self.get_section();
                    }
                    _ => {}
                }
            }
            None => {
                return;
            }
        }
        self.get_entity(entries);
    }

    fn get_section(&mut self) {
        self.bump();
        self.bump();

        self.skip_line_ws(true);

        self.get_keyword();


        self.skip_line_ws(true);

        self.bump();
        self.bump();
    }


    fn get_entity(&mut self, entries: &mut Map<String, Value>) {
        let id = self.get_identifier();

        self.skip_line_ws(true);

        if !self.peek_char_eq('=') {
            panic!();
        }

        self.bump();

        self.skip_line_ws(true);

        let value = self.get_pattern();

        match self.peek_char() {
            Some(&ch) => {
                if (ch == '[' && !self.peek_char_eq('[')) || ch == '*' {
                    self.source.reset_peek();

                    let (members, default_index) = self.get_members();

                    entries.insert(id,
                                   Value::ComplexValue {
                                       val: match value {
                                           Pattern::Simple(v) => {
                                               if v.is_empty() {
                                                   None
                                               } else {
                                                   Some(Pattern::Simple(v))
                                               }
                                           }
                                           Pattern::Complex(v) => Some(Pattern::Complex(v)),
                                       },
                                       traits: Some(members),
                                       def: default_index,
                                   });
                    return;
                } else {
                    self.source.reset_peek();
                }
            }
            None => {}
        }
        entries.insert(id, Value::Pattern(value));
    }

    fn get_members(&mut self) -> (Vec<Member>, Option<i8>) {
        let mut members = vec![];
        let mut default_index: Option<i8> = None;
        let mut index = 0;

        loop {
            match self.peek_char() {
                Some(&c) => {
                    let mut ch = c;
                    if ch == '*' {
                      self.bump();
                      default_index = Some(index);
                      match self.peek_char() {
                          Some(&c2) => ch = c2,
                          None => panic!() 
                      }
                    }

                    if ch != '[' || self.peek_char_eq('[') {
                        self.source.reset_peek();
                        break;
                    } else {
                      self.bump();
                    }

                    self.skip_line_ws(true);

                    let keyword = self.get_member_key();

                    self.skip_line_ws(true);

                    self.bump();

                    self.skip_line_ws(true);

                    let member = Member {
                        key: keyword,
                        val: self.get_pattern(),
                    };
                    members.push(member);
                    index += 1;

                    self.skip_ws()
                },
                None => break,
            }
        }

        (members, default_index)
    }

    fn get_comment(&mut self) {
        self.bump();
        if self.peek_char_eq(' ') {
            self.bump();
        }

        loop {
          match self.peek_char() {
            Some(&ch) => match ch {
              '\n' => {
                if !self.peek_char_eq('#') {
                  self.source.reset_peek();
                  break;
                }
                self.bump();
              },
              _ => {
                self.bump()
              },
            },
            None => break,
          }
        }
    }

    fn get_identifier(&mut self) -> String {
        let mut name = String::new();

        match self.read_char() {
            Some(ch) if is_id_start(ch) => name.push(ch),
            _ => panic!(),
        }

        loop {
            match self.peek_char() {
                Some(&ch) if is_id_char(ch) => name.push(ch),
                _ => break,
            }
            self.bump();
        }

        self.source.reset_peek();

        name
    }

    fn get_keyword(&mut self) -> Keyword {
        let mut name = String::new();
        let mut namespace = self.get_identifier();

        if self.peek_char_eq('/') {
            self.bump();
        } else {
            self.source.reset_peek();
            name = namespace;
            namespace = String::new();
        }

        match self.peek_char() {
            Some(&ch) if is_id_start(ch) => {
                self.bump();
                name.push(ch)
            }
            None => panic!(),
            _ => {
                if name.is_empty() {
                    panic!()
                }
                self.source.reset_peek();
            }
        }

        loop {
            match self.peek_char() {
                Some(&ch) if is_kw_char(ch) => name.push(ch),
                _ => {
                  self.source.reset_peek();
                  break;
                },
            }

            self.bump();
        }

        Keyword {
            t: String::from("kw"),
            name: name,
            ns: if namespace.is_empty() {
                None
            } else {
                Some(namespace)
            },
        }
    }

    fn get_pattern(&mut self) -> Pattern {
        let mut buffer = String::new();
        let mut content: Vec<PatternElement> = vec![];
        let mut quote_delimited: bool = false;
        let mut first_line = true;

        if self.peek_char_eq('"') {
            quote_delimited = true;
            self.bump();
        } else {
            self.source.reset_peek();
        }

        loop {
            match self.peek_char() {
                Some(&ch) => {
                    match ch {
                        '\n' => {
                            if quote_delimited {
                                panic!("Unclosed string");
                            }
                            if !self.first_non_ws_char_eq('|') {
                                self.source.reset_peek();
                                break;
                            }
                            if first_line && buffer.len() != 0 {
                                panic!("Multiline string should have the ID line empty");
                            }

                            if !first_line {
                                buffer.push(ch);
                            }
                            first_line = false;
                            self.bump();
                            if self.peek_char_eq(' ') {
                                self.bump();
                            } else {
                                self.source.reset_peek();
                            }
                            continue;
                        },
                        '\\' => {
                            self.bump();
                            match self.peek_char() {
                                Some(&ch2) => {
                                    if (quote_delimited && ch2 == '"') || ch2 == '{' {
                                        buffer.push(ch2);
                                        self.bump();
                                        continue;
                                    }
                                }
                                None => {}
                            }
                        },
                        '"' => {
                            self.bump();
                            quote_delimited = false;
                            break;
                        },
                        '{' => {
                            if buffer.len() != 0 {
                                content.push(
                                    PatternElement::TextElement(
                                        buffer.clone()
                                    )
                                );
                            }
                            buffer.clear();
                            
                            content.push(self.get_placeable());

                            continue;
                        },
                        _ => {
                            buffer.push(ch);
                        }
                    }
                }
                None => break,
            }
            self.bump();
        }

        if quote_delimited {
            panic!("Unclosed string");
        }

        if content.len() == 0 {
            Pattern::Simple(buffer)
        } else {
            if buffer.len() != 0 {
                content.push(PatternElement::TextElement(buffer.clone()));
            }
            Pattern::Complex(content)
        }

    }

    fn get_placeable(&mut self) -> PatternElement {
        let mut expressions = vec![];

        self.bump();
        self.skip_line_ws(true);

        loop {
            expressions.push(self.get_placeable_expression());

            self.skip_line_ws(true);

            match self.peek_char() {
                Some(&ch) => match ch {
                    '}' => {
                        self.bump();
                        break;
                    },
                    ',' => {
                        self.bump();
                        self.skip_ws();
                    },
                    _ => panic!(),
                },
              None => panic!(),
            }
        }

        PatternElement::PlaceableElement(expressions)
    }

    fn get_placeable_expression(&mut self) -> Expression {
        let selector = self.get_call_expression();

        self.skip_line_ws(true);

        match self.peek_char() {
            Some(&ch) => match ch {
                '-' => {
                    if self.peek_char_eq('>') {
                        self.bump();
                        self.bump();


                        self.skip_line_ws(true);

                        if !self.peek_char_eq('\n') {
                            panic!();
                        }

                        self.skip_ws();

                        let (members, default_index) = self.get_members();

                        return Expression::SelectExpression {
                            exp: Box::new(selector),
                            vars: members,
                            def: default_index
                        }
                    } else {
                        self.source.reset_peek()
                    }
                },
                _ => {
                    self.source.reset_peek();
                },
            },
            None => panic!(),
        }

        return selector;
    }

    fn get_call_expression(&mut self) -> Expression {
        let exp = self.get_member_expression();

        if !self.peek_char_eq('(') {
            self.source.reset_peek();
            return exp;
        }

        self.bump();

        let args = self.get_call_args();

        self.bump();

        Expression::CallExpression {
            name: Box::new(exp),
            args: args
        }
    }

    fn get_call_args(&mut self) -> Vec<Expression> {
        let mut args = vec![];

        self.skip_line_ws(true);

        loop {
            match self.peek_char() {
                Some(&ch) => match ch {
                    ')' => {
                        self.source.reset_peek();
                        break;
                    },
                    ',' => {
                        self.bump();
                        self.skip_line_ws(true);
                    },
                    _ => {
                        self.source.reset_peek();
                        let exp = self.get_call_expression();

                        self.skip_line_ws(true);


                        if self.peek_char_eq(':') {
                            self.bump();
                            self.skip_line_ws(true);

                            let val = self.get_call_expression();

                            match exp {
                                Expression::EntityReference(name) => {
                                    args.push(Expression::KeyValueArgument {
                                        name: name,
                                        val: Box::new(val)
                                    });
                                },
                                _ => {
                                    panic!();
                                }
                            }
                        } else {
                            self.source.reset_peek();
                            args.push(exp);
                        }
                    }
                },
                None => panic!()
            }
        }

        args
    }

    fn get_number(&mut self) -> Expression {
        let mut num = String::new();

        if self.peek_char_eq('-') {
            num.push('-');
            self.bump();
        } else {
            self.source.reset_peek();
        }

        match self.peek_char() {
            Some(&ch) => match ch {
                '0' ... '9' => {
                    num.push(ch);
                    self.bump();
                },
                _ => panic!(),
            },
            None => panic!(),
        }

        loop {
            match self.peek_char() {
                Some(&ch) => match ch {
                    '0' ... '9' => {
                        num.push(ch);
                    },
                    _ => {
                        self.source.reset_peek();
                        break;
                    },
                },
                None => {
                    self.source.reset_peek();
                    break;
                },
            }
            self.bump();
        }

        if self.peek_char_eq('.') {
            num.push('.');
            self.bump();

            match self.peek_char() {
                Some(&ch) => match ch {
                    '0' ... '9' => {
                        num.push(ch);
                        self.bump();
                    },
                    _ => panic!(),
                },
                None => panic!(),
            }

            loop {
                match self.peek_char() {
                    Some(&ch) => match ch {
                        '0' ... '9' => {
                            num.push(ch);
                        },
                        _ => {
                            self.source.reset_peek();
                            break;
                        },
                    },
                    None => {
                        self.source.reset_peek();
                        break;
                    },
                }
                self.bump();
            }

        } else {
            self.source.reset_peek();
        }

        Expression::Number(num)
    }

    fn get_member_expression(&mut self) -> Expression {
        let mut exp = self.get_literal();

        loop {
            if self.peek_char_eq('[') {
                self.bump();
                let keyword = self.get_member_key();
                self.bump();

                exp = Expression::Member {
                    obj: Box::new(exp),
                    key: keyword
                }
            } else {
                self.source.reset_peek();
                break;
            }
        }
        exp
    }

    fn get_member_key(&mut self) -> MemberKey {
        match self.peek_char() {
            Some(&ch) => match ch {
                '0'...'9' | '-' => {
                    let num = self.get_number();
                    match self.get_number() {
                        Expression::Number(val) => {
                            return MemberKey::Number(val);
                        },
                        _ => panic!()
                    }
                },
                _ => {
                    return MemberKey::Keyword(self.get_keyword());
                }
            },
            None => panic!()
        }
    }

    fn get_literal(&mut self) -> Expression {
        match self.peek_char() {
            Some(&ch) => match ch {
                '0' ... '9' | '-' => {
                    self.source.reset_peek();
                    return self.get_number();
                },
                '"' => {
                    let pat = self.get_pattern();
                    match pat {
                        Pattern::Simple(val) => {
                            return Expression::Pattern(val);
                        },
                        _ => {
                            panic!();
                        }
                    }
                },
                '$' => {
                    self.bump();
                    return Expression::ExternalArgument(
                        self.get_identifier()
                    )
                },
                _ => {
                    self.source.reset_peek();
                    return Expression::EntityReference(
                        self.get_identifier()
                    )
                }
            },
            None => panic!(),
        }
    }

    fn dump_ptr(&mut self) {
        println!("{:?}", self.peek_char().unwrap());
        println!("{:?}", self.read_char().unwrap());
    }
}
