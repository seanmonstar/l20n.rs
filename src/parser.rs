/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::num::FromStrRadix;

pub type ParseResult<T> = Result<T, ParseError>;

/// An error occurred trying to parse an L20n resource. The L20n file is
/// invalid.
#[deriving(Show)]
pub struct ParseError {
  /// The kind of error.
  pub kind: ParseErrorKind,
  /// The line where the error occurred.
  pub line: uint,
  /// The column where the error occurred.
  pub col: uint,
}

/// The description of the ParseError that occurred.
#[deriving(Show, PartialEq)]
pub enum ParseErrorKind {
  /// Illegal syntax for an identifier.
  IdentifierError,
  /// Illegal syntax for an entry.
  EntryError,
  /// Illegal syntax for an entity.
  EntityError,
  /// Illegal syntax for a macro.
  MacroError,
  /// Illegal syntax for an expression.
  ExprError,
  /// Illegal syntax for an operator.
  OpError,
  /// Illegal syntax for an expression wrapped in parenthesis.
  ParenError,
  /// Illegal syntax for an attribute.
  AttrError,
  /// Illegal syntax for a call expression (calling a macro).
  CallError,
  /// Illegal syntax for a value, when a value was expected.
  ValueError,
  /// Illegal syntax for a $var.
  VarError,
  /// Illegal syntax for a "String".
  StrError,
  /// Illegal syntax for a Hash.
  HashError,
}

#[deriving(Show, PartialEq, Clone)]
pub enum Entry {
  Entity(String, Value, Vec<Expr>, Vec<Attr>),
  Macro(String, Vec<Expr>, Expr),
  Import(String),
  Comment(String),
}

#[deriving(Show, PartialEq, Clone)]
pub enum Value {
  Str(String),
  ComplexStr(Vec<Expr>),
  Hash(HashMap<String, Value>, Option<String>, Option<Box<Expr>>)
}

#[deriving(Show, PartialEq, Clone)]
pub enum AccessType {
  Computed,
  Static,
}

#[deriving(Show, PartialEq, Clone)]
pub enum Expr {
  CondExpr(Box<Expr>, Box<Expr>, Box<Expr>),
  BinExpr(Box<Expr>, BinOp, Box<Expr>),
  UnExpr(UnOp, Box<Expr>),
  VarExpr(String),
  ValExpr(Value),
  PropExpr(Box<Expr>, Box<Expr>, AccessType),
  AttrExpr(Box<Expr>, Box<Expr>, AccessType),
  CallExpr(Box<Expr>, Vec<Expr>),
  IdentExpr(String),
  NumExpr(int),
  ParenExpr(Box<Expr>),
  GlobalExpr(String),
  ThisExpr,
}

#[deriving(Show, PartialEq, Clone)]
pub enum Attr {
  Attr(String, Value, Vec<Expr>)
}

#[deriving(Show, PartialEq, Clone)]
pub enum BinOp {
  BiAdd,
  BiSub,
  BiMul,
  BiDiv,
  BiRem,
  BiAnd,
  BiOr,
  BiEq,
  BiNe,
  BiLt,
  BiLe,
  BiGt,
  BiGe
}

#[deriving(Show, PartialEq, Clone)]
pub enum UnOp {
  UnAdd,
  UnSub,
  UnNot
}

pub struct Parser<T> {
  reader: T,
  ch: Option<char>,
  lookahead: Option<char>,
  line: uint,
  col: uint,
}

impl<T: Iterator<char>> Parser<T> {
  pub fn new(source: T) -> Parser<T> {
    Parser {
      reader: source,
      ch: None,
      lookahead: None,
      line: 0,
      col: 0,
    }
  }

  fn error(&self, kind: ParseErrorKind) -> ParseError {
    ParseError {
      kind: kind,
      line: self.line,
      col: self.col,
    }
  }

  fn bump(&mut self) {
    match self.lookahead.take() {
      None => { self.ch = self.reader.next(); }
      Some(ch) => { self.ch = Some(ch); }
    }

    match self.ch {
      Some(ch) => {
        if ch == '\n' {
          self.line += 1;
          self.col = 1;
        } else {
          self.col += 1;
        }
      }
      None => { }
    }
  }

  fn peek(&mut self) -> Option<char> {
    match self.lookahead {
      None => {
        self.lookahead = self.reader.next();
        self.lookahead
      }
      Some(ch) => Some(ch),
    }
  }

  fn ch_is(&self, ch: char) -> bool {
    self.ch == Some(ch)
  }

  pub fn parse(mut self) -> ParseResult<Vec<Entry>> {
    self.bump();
    let mut entries = vec!();
    loop {
      self.parse_whitespace();
      let ch = match self.ch {
        Some(ch) => ch,
        None => { break; }
      };

      if ch == '<' {
        entries.push(try!(self.parse_entry()));
      } else if ch == '/' && self.peek() == Some('*') {
        entries.push(try!(self.parse_comment()));
      } else {
        return Err(self.error(EntryError))
      }
    }
    Ok(entries)
  }

  fn parse_whitespace(&mut self) {
    while self.ch_is(' ') ||
          self.ch_is('\n') ||
          self.ch_is('\t') ||
          self.ch_is('\r') { self.bump(); }
  }

  fn parse_entry(&mut self) -> ParseResult<Entry> {
    self.bump();
    let id = try!(self.parse_identifier());
    let val = match self.ch {
      Some('(') => try!(self.parse_macro(id)),
      Some(_) => try!(self.parse_entity(id)),
      None => return Err(self.error(EntryError))
    };

    self.parse_whitespace();
    if self.ch_is('>') {
      self.bump();
      Ok(val)
    } else {
      Err(self.error(EntryError))
    }
  }

  fn parse_macro(&mut self, id: String) -> ParseResult<Entry> {
    if id.as_slice().char_at(0) == '_' {
      return Err(self.error(MacroError));
    }
    self.bump();
    self.parse_whitespace();

    let mut args = vec![];

    try!(self.parse_list(')', MacroError, |this| {
      args.push(try!(this.parse_variable()));
      Ok(())
    }));

    self.bump();
    self.parse_whitespace();

    if !self.ch_is('{') {
      return Err(self.error(MacroError));
    }
    self.bump();
    self.parse_whitespace();
    
    let body = try!(self.parse_expression());

    self.parse_whitespace();
    
    if !self.ch_is('}') {
      return Err(self.error(MacroError));
    }
    self.bump();

    Ok(Macro(id, args, body))
  }

  fn parse_entity(&mut self, id: String) -> ParseResult<Entry> {
    let mut index = vec![];
    if self.ch_is('[') {
      self.bump();
      try!(self.parse_list(']', EntityError, |this| {
        index.push(try!(this.parse_expression()));
        Ok(())
      }));
      self.bump();
    }

    // whitespace here is required
    if !self.ch_is(' ') {
      return Err(self.error(EntityError));
    };
    self.parse_whitespace();

    let value = try!(self.parse_value());
    self.parse_whitespace();
    let attrs = try!(self.parse_attrs());

    Ok(Entity(id, value, index, attrs))
  }

  fn parse_attrs(&mut self) -> ParseResult<Vec<Attr>> {
    let mut attrs = vec![];
    loop {
      match self.ch {
        Some('>') => break,
        _ => {}
      }

      let id = try!(self.parse_identifier());

      let mut indices = vec![];
      if self.ch_is('[') {
        self.bump();
        try!(self.parse_list(']', AttrError, |this| {
          indices.push(try!(this.parse_expression()));
          Ok(())
        }));
      }

      self.parse_whitespace();
      if !self.ch_is(':') {
        return Err(self.error(AttrError));
      }
      self.bump();
      self.parse_whitespace();

      let value = try!(self.parse_value());

      attrs.push(Attr(id, value, indices));
    }
    Ok(attrs)
  }

  fn parse_comment(&mut self) -> ParseResult<Entry> {
    self.bump();
    self.bump();
    let mut s = String::new();
    loop {
      match self.ch {
        Some(c@'*') => {
          if self.peek() == Some('/') {
            self.bump();
            self.bump();
            break;
          } else {
            s.push_char(c);
          }
        },
        Some(c) => s.push_char(c),
        None => return Err(self.error(EntryError)),
      }
      self.bump();
    }
    Ok(Comment(s))
  }

  fn parse_value(&mut self) -> ParseResult<Value> {
    match self.ch {
      Some('"') | Some('\'') => self.parse_str(),
      Some('{') => self.parse_hash(),
      _ => Err(self.error(ValueError))
    }
  }

  fn parse_str(&mut self) -> ParseResult<Value> {
    let mut s = String::new();

    let quote = self.ch.unwrap();
    let mut exprs = vec![];

    loop {
      self.bump();
      match self.ch {
        Some(c@'{') => {
          if self.peek() == Some('{') {
            self.bump();
            self.bump();
            self.parse_whitespace();
            let expr = try!(self.parse_expression());
            self.parse_whitespace();
            if self.ch_is('}') && self.peek() == Some('}') {
              self.bump();
              exprs.push(ValExpr(Str(s)));
              exprs.push(expr);
              s = String::new();
            } else {
              return Err(self.error(ValueError));
            }
          } else {
            s.push_char(c);
          }
        },
        Some('\\') => {},
        Some(c) if c == quote => { self.bump(); break },
        Some(c) => s.push_char(c),
        None => return Err(self.error(StrError))
      }
    }

    if exprs.len() > 0 {
      if s.len() > 0 {
        exprs.push(ValExpr(Str(s)));
      }
      Ok(ComplexStr(exprs))
    } else {
      Ok(Str(s))
    }
  }

  fn parse_hash(&mut self) -> ParseResult<Value> {
    self.bump();
    self.parse_whitespace();

    let mut map = HashMap::new();

    let mut default = None;

    try!(self.parse_list('}', HashError, |this| {
      let mut is_default = false;


      match this.ch {
        Some('*') => {
          match default {
            Some(_) => return Err(this.error(HashError)),
            None => {
              is_default = true;
              this.bump()
            }
          }
        },
        Some(_) => {},
        None => return Err(this.error(HashError)),
      }
      let key = try!(this.parse_identifier());
      if is_default {
        default = Some(key.clone());
      }

      this.parse_whitespace();

      match this.ch {
        Some(':') => { this.bump() },
        _ => return Err(this.error(HashError))
      }

      this.parse_whitespace();
      let value = try!(this.parse_value());
      this.parse_whitespace();

      map.insert(key, value);
      Ok(())
    }));

    self.bump();
    self.parse_whitespace();

    Ok(Hash(map, default, None))
  }

  fn parse_list(&mut self, end: char, err: ParseErrorKind, handle: |&mut Parser<T>| -> ParseResult<()>) -> ParseResult<()> {
    loop {
      try!(handle(self));

      match self.ch {
        Some(',') => {
          self.bump();
          self.parse_whitespace();
        },
        Some(c) if c == end => break,
        _ => return Err(self.error(err))
      }
    }
    Ok(())
  }

  fn parse_expression(&mut self) -> ParseResult<Expr> {
    self.parse_cond_expression()
  }

  fn parse_cond_expression(&mut self) -> ParseResult<Expr> {
    let cond = try!(self.parse_or_expression());
    self.parse_whitespace();

    match self.ch {
      Some('?') => { self.bump() },
      _ => return Ok(cond),
    }

    self.parse_whitespace();
    let consequent = try!(self.parse_expression());
    self.parse_whitespace();
    if !self.ch_is(':') {
      return Err(self.error(ExprError))
    }
    self.bump();
    self.parse_whitespace();
    let alternate = try!(self.parse_expression());

    Ok(CondExpr(box cond, box consequent, box alternate))
  }

  fn parse_prefix_expression(&mut self, ops: &[BinOp], next: |&mut Parser<T>| -> ParseResult<Expr>) -> ParseResult<Expr> {
    let mut exp = try!(next(self));
    loop {
      self.parse_whitespace();
      let mut binop = None;
      for op in ops.iter() {
        let mut chars = self.peek_bin_op(op);
        if chars > 0 {
          binop = Some(*op);
          while chars > 0 {
            self.bump();
            chars -= 1;
          }
          break;
        }
      }
      let binop = match binop {
        None => break,
        Some(op) => op
      };

      self.parse_whitespace();
      let right = try!(next(self));
      exp = BinExpr(box exp, binop, box right);
    }
    Ok(exp)
  }

  fn parse_postfix_expression(&mut self, ops: &[UnOp], next: |&mut Parser<T>| -> ParseResult<Expr>) -> ParseResult<Expr> {
    let mut unop = None;
    for op in ops.iter() {
      if self.peek_un_op(op) {
        unop = Some(*op);
        break;
      }
    }

    let unop = match unop {
      None => return next(self),
      Some(op) => op
    };
    self.bump();
    self.parse_whitespace();

    Ok(UnExpr(unop, box try!(self.parse_postfix_expression(ops, next))))
  }

  fn peek_bin_op(&mut self, op: &BinOp) -> uint {
    let (c1, c2) = match (self.ch, self.peek()) {
      (Some(c1), Some(c2)) => (c1, c2),
      _ => return 0
    };

    match (*op, c1, c2) {
      (BiAnd, '&', '&') |
      (BiOr, '|', '|')  |
      (BiEq, '=', '=')  |
      (BiNe, '!', '=')  |
      (BiGe, '>', '=')  |
      (BiLe, '<', '=') => 2,
      (BiGt, '>', _)    |
      (BiLt, '<', _)    |
      (BiAdd, '+', _)   |
      (BiSub, '-', _)   |
      (BiMul, '*', _)   |
      (BiDiv, '/', _)   |
      (BiRem, '%', _)  => 1,
      _ => 0
    }
  }

  fn peek_un_op(&self, op: &UnOp) -> bool {
    match (self.ch, *op) {
      (Some('+'), UnAdd) |
      (Some('-'), UnSub) |
      (Some('!'), UnNot) => true,
      _ => false
    }
  }


  fn parse_or_expression(&mut self) -> ParseResult<Expr> {
    self.parse_prefix_expression([BiOr], |this| this.parse_and_expression())
  }

  fn parse_and_expression(&mut self) -> ParseResult<Expr> {
    self.parse_prefix_expression([BiAnd], |this| this.parse_eq_expression())
  }

  fn parse_eq_expression(&mut self) -> ParseResult<Expr> {
    self.parse_prefix_expression([BiEq, BiNe], |this| this.parse_rel_expression())
  }

  fn parse_rel_expression(&mut self) -> ParseResult<Expr> {
    self.parse_prefix_expression([BiGe, BiGt, BiLe, BiLt], |this| this.parse_add_expression())
  }

  fn parse_add_expression(&mut self) -> ParseResult<Expr> {
    self.parse_prefix_expression([BiAdd, BiSub], |this| this.parse_rem_expression())
  }

  fn parse_rem_expression(&mut self) -> ParseResult<Expr> {
    self.parse_prefix_expression([BiRem], |this| this.parse_mul_expression())
  }

  fn parse_mul_expression(&mut self) -> ParseResult<Expr> {
    self.parse_prefix_expression([BiMul, BiDiv], |this| this.parse_unary_expression())
  }

  fn parse_unary_expression(&mut self) -> ParseResult<Expr> {
    self.parse_postfix_expression([UnAdd, UnSub, UnNot], |this| this.parse_member_expression())
  }

  fn parse_member_expression(&mut self) -> ParseResult<Expr> {
    let mut exp = try!(self.parse_paren_expression());

    loop {
      match self.ch {
        Some('.') | Some('[') => {
          exp = try!(self.parse_property_expression(exp));
        },
        Some(':') => {
          if self.peek() == Some(':') {
            self.bump();
            self.bump();
            exp = try!(self.parse_attr_expression(exp));
          } else {
            break;
          }
        },
        Some('(') => exp = try!(self.parse_call_expression(exp)),
        _ => break
      }
    }

    Ok(exp)
  }

  fn parse_property_expression(&mut self, accessed: Expr) -> ParseResult<Expr> {
    let computed = self.ch_is('[');
    self.bump();
    if computed {
      self.parse_whitespace();
      let exp = try!(self.parse_expression());
      self.parse_whitespace();
      if !self.ch_is(']') {
        return Err(self.error(ExprError));
      }
      self.bump();
      Ok(PropExpr(box accessed, box exp, Computed))
    } else {
      let exp = try!(self.parse_identifier());
      Ok(PropExpr(box accessed, box IdentExpr(exp), Static))
    }
  }

  fn parse_attr_expression(&mut self, accessed: Expr) -> ParseResult<Expr> {
    match accessed {
      ParenExpr(..) | IdentExpr(..) | ThisExpr => {},
      _ => return Err(self.error(AttrError))
    }
    let computed = self.ch_is('[');

    if computed {
      self.bump();
      self.parse_whitespace();
      let exp = try!(self.parse_expression());
      self.parse_whitespace();
      if !self.ch_is(']') {
        return Err(self.error(ExprError))
      }
      self.bump();

      Ok(AttrExpr(box accessed, box exp, Computed))
    } else {
      Ok(AttrExpr(box accessed, box try!(self.parse_expression()), Static))
    }
  }

  fn parse_call_expression(&mut self, callee: Expr) -> ParseResult<Expr> {
    self.bump(); // (
    let mut args = vec![];

    try!(self.parse_list(')', CallError, |this| {
      args.push(try!(this.parse_expression()));
      Ok(())
    }));
    self.bump(); // )

    Ok(CallExpr(box callee, args))
  }

  fn parse_paren_expression(&mut self) -> ParseResult<Expr> {
    match self.ch {
      Some('(') => self.bump(),
      _ => return self.parse_primary_expression()
    }

    self.parse_whitespace();
    let exp = try!(self.parse_expression());
    self.parse_whitespace();

    match self.ch {
      Some(')') => self.bump(),
      _ => return Err(self.error(ParenError))
    }

    Ok(ParenExpr(box exp))
  }

  fn parse_primary_expression(&mut self) -> ParseResult<Expr> {
    match self.ch {
      Some(c) => {
        match c {
          '0'..'9' => self.parse_number(),
          '\'' | '"' | '{' | '[' => Ok(ValExpr(try!(self.parse_value()))),
          '$' => self.parse_variable(),
          '@' => {
            self.bump();
            Ok(GlobalExpr(try!(self.parse_identifier())))
          },
          '~' => {
            self.bump();
            Ok(ThisExpr)
          },
          _ => Ok(IdentExpr(try!(self.parse_identifier())))
        }
      },
      None => Err(self.error(ExprError))
    }
  }

  fn parse_number(&mut self) -> ParseResult<Expr> {
    let mut num = String::new();
    loop {
      match self.ch {
        Some(ch) => match ch {
          '0'..'9' => {
            num.push_char(ch);
            self.bump();
          },
          _ => break
        },
        _ => break,
      }
    }

    if num.len() > 0 {
      Ok(NumExpr(FromStrRadix::from_str_radix(num.as_slice(), 10).unwrap()))
    } else {
      Err(self.error(ExprError))
    }
  }

  fn parse_variable(&mut self) -> ParseResult<Expr> {
    if !self.ch_is('$') {
      return Err(self.error(VarError));
    }
    self.bump();
    Ok(VarExpr(try!(self.parse_identifier())))
  }

  fn parse_identifier(&mut self) -> ParseResult<String> {
    let mut id = String::new();
    // identifiers must start with a-zA-Z_
    match self.ch {
      Some(c) => match c {
        'a'..'z' | 'A'..'Z' | '_' => id.push_char(c),
        _ => return Err(self.error(IdentifierError))
      },
      None => return Err(self.error(IdentifierError))
    }

    loop {
      self.bump();
      let ch = match self.ch {
        Some(c) => c,
        None => return Err(self.error(IdentifierError))
      };

      match ch {
        'a'..'z' | 'A'..'Z' | '0'..'9' | '_' => id.push_char(ch),
        _ => break,
      }
    }
    Ok(id)
  }
}

#[cfg(test)]
mod tests {
  use super::{Parser, Entity, Str, Hash, Attr, VarExpr, Macro, CondExpr,
              BinExpr, ValExpr, ComplexStr, NumExpr, BiGt, BiGe, Comment};
  use std::collections::HashMap;

  fn s(v: &'static str) -> String {
    String::from_str(v)
  }

  #[test]
  fn test_basic_entity() {
    let p = Parser::new("<hello \"Hello, World\" >".chars());
    assert_eq!(p.parse().unwrap(), vec![
               Entity(s("hello"), Str(s("Hello, World")), vec![], vec![])
    ]);
  }

  #[test]
  fn test_multiple_entities() {
    let p = Parser::new("<hell0 \"Hello, World\">\n<bye 'Bye!'>".chars());
    assert_eq!(p.parse().unwrap(), vec![
               Entity(s("hell0"), Str(s("Hello, World")), vec![], vec![]),
               Entity(s("bye"), Str(s("Bye!")), vec![], vec![])
    ]);
  }

  #[test]
  fn test_macro() {
    let p = Parser::new("<foo($n) { $n > 1 ? 'foo' : 'bar' }>".chars());
    assert_eq!(p.parse().unwrap(), vec![
               Macro(s("foo"),
                     vec![VarExpr(s("n"))],
                     CondExpr(box BinExpr(box VarExpr(s("n")), BiGt, box NumExpr(1)),
                              box ValExpr(Str(s("foo"))),
                              box ValExpr(Str(s("bar")))
                     )
               ),
    ]);
  }

  #[test]
  fn test_ge() {
    let p = Parser::new("<foo($n) { $n >= 1 ? 'foo' : 'bar' }>".chars());
    assert_eq!(p.parse().unwrap(), vec![
               Macro(s("foo"),
                     vec![VarExpr(s("n"))],
                     CondExpr(box BinExpr(box VarExpr(s("n")), BiGe, box NumExpr(1)),
                              box ValExpr(Str(s("foo"))),
                              box ValExpr(Str(s("bar")))
                     )
               ),
    ]);
  }

  #[test]
  fn test_hash() {
    let p = Parser::new("<pro { masculine: 'his', feminine: 'her'}>".chars());
    let mut map = HashMap::new();
    map.insert(s("masculine"), Str(s("his")));
    map.insert(s("feminine"), Str(s("her")));
    assert_eq!(p.parse().unwrap(), vec![
               Entity(s("pro"), Hash(map, None, None), vec![], vec![])
    ]);
  }

  #[test]
  fn test_hash_default() {
    let p = Parser::new("<pro { *masculine: 'his', feminine: 'her'}>".chars());
    let mut map = HashMap::new();
    map.insert(s("masculine"), Str(s("his")));
    map.insert(s("feminine"), Str(s("her")));
    assert_eq!(p.parse().unwrap(), vec![
               Entity(s("pro"), Hash(map, Some(s("masculine")), None), vec![], vec![])
    ]);
  }

  #[test]
  fn test_hash_index() {
    let p = Parser::new("<pro['feminine'] { masculine: 'his', feminine: 'her'}>".chars());
    let mut map = HashMap::new();
    map.insert(s("masculine"), Str(s("his")));
    map.insert(s("feminine"), Str(s("her")));
    assert_eq!(p.parse().unwrap(), vec![
               Entity(s("pro"), Hash(map, None, None), vec![ValExpr(Str(s("feminine")))], vec![])
    ]);
  }

  #[test]
  fn test_attr() {
    let p = Parser::new("<pro 'her' neuter: 'their'>".chars());
    assert_eq!(p.parse().unwrap(), vec![
               Entity(s("pro"), Str(s("her")), vec![], vec![Attr(s("neuter"), Str(s("their")), vec![])])
    ]);
  }

  #[test]
  fn test_complex_str() {
    let p = Parser::new("<hi 'Hello, {{ $name }}!'>".chars());
    assert_eq!(p.parse().unwrap(), vec![
               Entity(s("hi"), ComplexStr(vec![
                 ValExpr(Str(s("Hello, "))),
                 VarExpr(s("name")),
                 ValExpr(Str(s("!")))
                 ]), vec![], vec![])
               ]);
  }

  #[test]
  fn test_comment() {
    let p = Parser::new("/* foo bar */".chars());
    assert_eq!(p.parse().unwrap(), vec![Comment(s(" foo bar "))])
  }

}

