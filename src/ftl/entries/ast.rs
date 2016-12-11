extern crate serde;
extern crate serde_json;

use self::serde_json::Map;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource(pub Map<String, Value>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyword {
  pub t: String,
  pub name: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
  pub key: Keyword,
  pub val: Pattern
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Pattern(Pattern),
    ComplexValue {
      traits: Vec<Member>,
      val: Pattern
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub value: Option<Pattern>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub source: String,
}

