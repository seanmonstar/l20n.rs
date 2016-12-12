extern crate serde;
extern crate serde_json;

use self::serde::ser::Serializer;
use self::serde::ser::Serialize;


use self::serde_json::Map;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource(pub Map<String, Value>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyword {
  #[serde(rename="type")]
  pub t: String,
  pub name: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
  pub key: Keyword,
  pub val: Pattern
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Value {
    Pattern(Pattern),
    ComplexValue {
      traits: Vec<Member>,
      val: Pattern
    }
}

impl Serialize for Value {
  fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer
  {
    match self {
      &Value::Pattern( Pattern { ref source } ) => serializer.serialize_str(&source),
      &Value::ComplexValue { ref val, ref traits } => {
        let num_fields = if val.source.is_empty() { 1 } else { 2 };
        let mut map = serializer.serialize_map(Some(num_fields)).unwrap();
        if num_fields == 2 {
            serializer.serialize_map_key(&mut map, "val");
            serializer.serialize_map_value(&mut map, &val.source);
        }
        serializer.serialize_map_key(&mut map, "traits");
        serializer.serialize_map_value(&mut map, traits);
        serializer.serialize_map_end(map)
      }
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub value: Option<Pattern>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Pattern {
    pub source: String,
}

impl Serialize for Pattern {
  fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer
  {
    serializer.serialize_str(&self.source)
  }
}

