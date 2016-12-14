extern crate serde;
extern crate serde_json;
extern crate void;

use self::serde::ser::Serializer;
use self::serde::ser::Serialize;
use self::serde::de::Visitor;
use self::serde::de::MapVisitor;
use self::serde::de::Error;
use self::serde::de::{Deserialize, Deserializer};
use self::void::Void;
use std::str::FromStr;


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

#[derive(Debug, PartialEq)]
pub enum Value {
    Pattern(Pattern),
    ComplexValue {
      traits: Option<Vec<Member>>,
      val: Option<Pattern>
    }
}

impl Serialize for Value {
  fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where S: Serializer
  {
    match self {
      &Value::Pattern( Pattern { ref source } ) => serializer.serialize_str(&source),
      &Value::ComplexValue { ref val, ref traits } => {
        let num_fields = if !val.is_none() { 1 } else { 2 };
        let mut map = serializer.serialize_map(Some(num_fields)).unwrap();
        if let &Some(ref v) = val {
            try!(serializer.serialize_map_key(&mut map, "val"));
            try!(serializer.serialize_map_value(&mut map, &v.source));
        }
        try!(serializer.serialize_map_key(&mut map, "traits"));
        try!(serializer.serialize_map_value(&mut map, traits));
        serializer.serialize_map_end(map)
      }
    }
  }
}


impl Deserialize for Value {
    fn deserialize<D>(deserializer: &mut D) -> Result<Value, D::Error>
      where D: Deserializer
    {
      struct FieldVisitor;

      impl Visitor for FieldVisitor {
        type Value = Value;

        fn visit_str<E>(&mut self, value: &str) -> Result<Value, E>
          where E: Error
        {
          Ok(Value::Pattern(Pattern { source: String::from(value) }))
        }

        fn visit_map<V>(&mut self, mut visitor: V) -> Result<Value, V::Error>
          where V: MapVisitor
        {
          let mut val: Option<Pattern> = None;
          let mut traits: Option<Vec<Member>> = None;
          while let Some(key) = try!(visitor.visit_key()) {
            let key: String = key;
            match &key as &str {
              "val" => {
                let value: String = try!(visitor.visit_value());
                val = Some(Pattern { source: value });
              },
              "traits" => {
                let value: Vec<Member> = try!(visitor.visit_value());
                traits = Some(value);
              },
              _ => {}
            }
          }
          try!(visitor.end());
          Ok(Value::ComplexValue{
            val: val,
            traits: traits
          })
        }
      }
      deserializer.deserialize_struct_field(FieldVisitor)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub value: Value,
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

impl FromStr for Pattern {
  type Err = Void;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Pattern {
      source: s.to_string()
    })
  }
}
