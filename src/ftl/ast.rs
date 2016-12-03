extern crate rustc_serialize;

use self::rustc_serialize::json::{Json, ToJson};

pub enum Entry {
    Comment { content: String },
    Entity {
        id: Identifier,
        value: Value,
        traits: Option<Vec<Member>>,
    },
    Section { key: Keyword, body: Vec<Entry> },
}

pub struct Identifier {
    pub name: String,
}

impl ToJson for Identifier {
  fn to_json(&self) -> Json {
    self.name.to_json()
  }
}


pub struct Keyword {
    pub name: String,
}

pub struct Member {
    pub key: Keyword,
    pub value: Value,
    pub default: bool,
}

pub struct Expression {
    pub node: ExpressionKind,
}

pub enum ExpressionKind {
    IdentifierExpression { name: String },
}

pub enum PatternElement {
    TextElement { value: String },
    Placeable { expressions: Vec<Expression> },
}

pub enum Value {
    Pattern {
        source: String,
        elements: Vec<PatternElement>,
    },
}
