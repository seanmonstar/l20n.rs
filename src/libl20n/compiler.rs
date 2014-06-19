
use std::collections::HashMap;
use std::num::ToStrRadix;

use data;
use parser::{ParseError, Parser};
use parser;

#[deriving(Show)]
pub enum CompileError {
  ParserError(ParseError)
}

pub type CompileResult = Result<HashMap<String, parser::Entry>, CompileError>;

pub struct Compiler;

impl Compiler {
  pub fn new() -> Compiler {
    Compiler
  }

  pub fn compile(&mut self, source: &str) -> CompileResult {
    let p = Parser::new(source.chars());
    let entries = match p.parse() {
      Ok(entries) => entries,
      Err(e) => return Err(ParserError(e))
    };

    // TODO: parse all imports

    let mut map = HashMap::new();

    for mut entry in entries.move_iter() {
      let id = match entry {
        parser::Comment(..) | parser::Import(..) => continue,
        parser::Macro(ref id, _, _) => id.clone(),
        parser::Entity(ref id, ref mut value, ref indices)  => {
          // while we're here, fix up and Hash values with default indices
          match value  {
            &parser::Hash(..) => {
              if indices.is_some() {
                add_default_indices(value, indices.get_ref().as_slice());
              }
              id.clone()
            },
            _ => id.clone()
          }
        }
      };
      map.insert(id, entry);
    }

    Ok(map)
  }
}


fn add_default_indices(value: &mut parser::Value, mut indices: &[parser::Expr]) {
  match value {
    &parser::Hash(ref mut map, _, ref mut def_index) => {
      match indices.shift_ref() {
        Some(idx) => {
          for (_k, v) in map.mut_iter() {
            add_default_indices(v, indices);
          }
          *def_index = Some(box idx.clone())
        },
        None => {}
      }
    },
    _ => {}
  }
}


pub fn compile(source: &str) -> CompileResult {
  let mut c = Compiler;
  c.compile(source)
}


pub type Env = HashMap<String, parser::Entry>;

pub struct ResolveContext<'a> {
  data: &'a data::Data,
  env: &'a Env,
  locals: Option<&'a data::Data>,
  index: Option<String>,
}

impl<'a> ResolveContext<'a> {
  pub fn new(env: &'a Env, data: &'a data::Data) -> ResolveContext<'a> {
    ResolveContext {
      env: env,
      data: data,
      locals: None,
      index: None,
    }
  }

  fn with_locals(&'a self, locals: &'a data::Data) -> ResolveContext<'a> {
    ResolveContext {
      env: self.env,
      data: self.data,
      locals: Some(locals),
      index: None,
    }
  }

  fn with_index(&'a self, index: Option<String>) -> ResolveContext<'a> {
    ResolveContext {
      env: self.env,
      data: self.data,
      locals: self.locals,
      index: index,
    }
  }
}

pub type ResolveResult = Result<ResolveTarget, ResolveError>;

pub enum ResolveTarget {
  Entry(parser::Entry),
  Value(parser::Value),
  Data(data::Data)
}

#[deriving(Show)]
pub enum ResolveError {
  WrongType,
  WrongNumberOfArgs,
  MissingIndex,
  MissingVar(String),
  MissingIdent(String),
}

pub trait Resolve {
  fn resolve(&self, ctx: &ResolveContext) -> ResolveResult;

  fn resolve_data(&self, ctx: &ResolveContext) -> Result<data::Data, ResolveError> {
    match self.resolve(ctx) {
      Ok(Data(d)) => Ok(d),
      Ok(other) => other.resolve_data(ctx),
      Err(e) => Err(e)
    }
  }
}

impl Resolve for ResolveTarget {
  fn resolve(&self, ctx: &ResolveContext) -> ResolveResult {
    match *self {
      Data(ref d) => Ok(Data(d.clone())),
      Entry(ref e) => e.resolve(ctx),
      Value(ref v) => v.resolve(ctx)
    }
  }
}

impl Resolve for parser::Entry {
  fn resolve(&self, _ctx: &ResolveContext) -> ResolveResult {
    match *self {
      parser::Entity(_, ref value, _) => {
        Ok(Value(value.clone()))
      }
      _ => Ok(Data(data::Null))
    }
  }
}

impl Resolve for parser::Value {
  fn resolve(&self, ctx: &ResolveContext) -> ResolveResult {
    match *self {
      parser::Str(ref s) => Ok(Data(data::Str(s.clone()))),
      parser::ComplexStr(ref exprs) => {
        let mut vec = Vec::with_capacity(exprs.len());
        for expr in exprs.iter() {
          vec.push(match expr.resolve_data(ctx) {
            Ok(data::Str(s)) => s,
            Ok(data::Num(n)) => n.to_str_radix(10),
            Ok(_) => return Err(WrongType),
            Err(e) => return Err(e)
          });
        }
        Ok(Data(data::Str(vec.concat())))
      }
      parser::Hash(ref map, ref def_key, ref def_index) => {
        match ctx.index {
          Some(ref s) => match map.find(s) {
              Some(v) => return Ok(Value(v.clone())),
              None => {}
          },
          None => {}
        };
        match *def_key {
          Some(ref s) => match map.find(s) {
              Some(v) => return Ok(Value(v.clone())),
              None => {}
          },
          None => {}
        };
        match *def_index {
          Some(ref e) => match e.resolve_data(ctx) {
              Ok(data::Str(ref s)) => match map.find(s) {
                Some(v) => return Ok(Value(v.clone())),
                None => {}
              },
              Ok(_) => return Err(WrongType),
              Err(e) => return Err(e)
          },
          None => {}
        };
        Err(MissingIndex)
      }
    }
  }
}

impl Resolve for parser::Expr {
  fn resolve(&self, ctx: &ResolveContext) -> ResolveResult {
    match *self {
      parser::ValExpr(ref val) => Ok(Value(val.clone())),
      parser::NumExpr(ref n) => Ok(Data(data::Num(*n))),
      parser::BinExpr(ref left, ref op, ref right) => {
        let left = try!(left.resolve_data(ctx));
        let right = try!(right.resolve_data(ctx));
        match (*op, left, right) {
          // math ops
          (parser::BiAdd, data::Num(l), data::Num(r)) => Ok(Data(data::Num(l + r))),
          (parser::BiSub, data::Num(l), data::Num(r)) => Ok(Data(data::Num(l - r))),
          (parser::BiMul, data::Num(l), data::Num(r)) => Ok(Data(data::Num(l * r))),
          (parser::BiDiv, data::Num(l), data::Num(r)) => Ok(Data(data::Num(l / r))),
          (parser::BiRem, data::Num(l), data::Num(r)) => Ok(Data(data::Num(l % r))),

          (parser::BiLt, data::Num(l), data::Num(r)) => Ok(Data(data::Bool(l < r))),
          (parser::BiLe, data::Num(l), data::Num(r)) => Ok(Data(data::Bool(l <= r))),
          (parser::BiGt, data::Num(l), data::Num(r)) => Ok(Data(data::Bool(l > r))),
          (parser::BiGe, data::Num(l), data::Num(r)) => Ok(Data(data::Bool(l >= r))),

          // logical ops
          (parser::BiAnd, data::Bool(l), data::Bool(r)) => Ok(Data(data::Bool(l && r))),
          (parser::BiOr, data::Bool(l), data::Bool(r)) => Ok(Data(data::Bool(l || r))),

          // equality ops. can be Num, Bool, or Str
          (parser::BiEq, data::Bool(l), data::Bool(r)) => Ok(Data(data::Bool(l == r))),
          (parser::BiEq, data::Str(l), data::Str(r)) => Ok(Data(data::Bool(l == r))),
          (parser::BiEq, data::Num(l), data::Num(r)) => Ok(Data(data::Bool(l == r))),
          (parser::BiNe, data::Bool(l), data::Bool(r)) => Ok(Data(data::Bool(l == r))),
          (parser::BiNe, data::Str(l), data::Str(r)) => Ok(Data(data::Bool(l == r))),
          (parser::BiNe, data::Num(l), data::Num(r)) => Ok(Data(data::Bool(l != r))),

          (_, _, _) => Err(WrongType)
        }
      }
      parser::UnExpr(ref op, ref expr) => {
        let expr = try!(expr.resolve_data(ctx));
        match (*op, expr) {
          (parser::UnAdd, data::Num(n)) => Ok(Data(data::Num(n))),
          (parser::UnSub, data::Num(n)) => Ok(Data(data::Num(-n))),
          (parser::UnNot, data::Bool(b)) => Ok(Data(data::Bool(!b))),
          _ => Err(WrongType)
        }
      }
      parser::VarExpr(ref name) => {
        match ctx.locals.and_then(|locals| locals.find_copy(name)) {
          Some(val) => return Ok(Data(val)),
          _ => {}
        };
        match ctx.data.find_copy(name) {
          Some(d) => Ok(Data(d)),
          None => Err(MissingVar(name.clone()))
        }
      }
      parser::IdentExpr(ref ident) => {
        match ctx.env.find(ident) {
          Some(e) => e.resolve(ctx),
          None => Err(MissingIdent(ident.clone()))
        }
      }
      parser::CondExpr(ref cond, ref consequent, ref alt) => {
        match try!(cond.resolve_data(ctx)) {
          data::Bool(b) => {
            if b {
              consequent.resolve(ctx)
            } else {
              alt.resolve(ctx)
            }
          },
          _ => Err(WrongType)
        }
      }
      parser::CallExpr(box parser::IdentExpr(ref ident), ref args) => {
        match ctx.env.find(ident) {
          Some(&parser::Macro(_, ref arg_names, ref body)) => {
            if args.len() == arg_names.len() {
              let mut map = HashMap::new();
              for (k, v) in arg_names.iter().zip(args.iter()) {
                let name = match k {
                  &parser::VarExpr(ref name) => name.clone(),
                  // not a VarExpr would be the parser going nuts
                  _ => unreachable!()
                };
                let arg = match v.resolve_data(ctx) {
                  Ok(val) => val,
                  Err(e) => return Err(e)
                };
                map.insert(name, arg);
              }
              let locals = data::Map(map);
              body.resolve(&ctx.with_locals(&locals))
            } else {
              Err(WrongNumberOfArgs)
            }
          }
          Some(_) => Err(WrongType),
          None => Err(MissingIdent(ident.clone()))
        }
      }
      parser::PropExpr(ref parent, ref prop, ref access) => {
        let prop = match *access {
          parser::Computed => match prop.resolve_data(ctx) {
            Ok(data::Str(s)) => s,
            Ok(_) => return Err(WrongType),
            Err(e) => return Err(e)
          },
          parser::Static => match *prop {
            box parser::IdentExpr(ref s) => s.clone(),
            _ => return Err(WrongType)
          }
        };

        println!("PropExpr {} {}", parent, prop);
        match parent.resolve(ctx) {
          Ok(Data(data::Map(ref m))) => {
            match m.find_copy(&prop) {
              Some(d) => Ok(Data(d)),
              None => Err(MissingIndex)
            }
          },
          Ok(Value(ref v)) => {
            v.resolve(&ctx.with_index(Some(prop)))
          },
          Ok(_) => Err(WrongType),
          Err(e) => Err(e)
        }
      }
      ref e => fail!("{} not yet implemented", e)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{Compiler, Resolve, ResolveContext};
  use data::{Str, Null};

  #[test]
  fn test_compile() {
    let mut c = Compiler;
    let map = c.compile("<hi 'hello world'>").unwrap();
    let entity = map.get(&String::from_str("hi"));
    let data = Null;
    let ctx = ResolveContext::new(&map, &data);

    assert_eq!(entity.resolve_data(&ctx).unwrap(), Str(String::from_str("hello world")));

  }
}
