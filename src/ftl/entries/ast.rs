extern crate serde;
extern crate serde_json;

use self::serde::ser::Serializer;
use self::serde::ser::Serialize;
use self::serde::de::Visitor;
use self::serde::de::MapVisitor;
use self::serde::de::SeqVisitor;
use self::serde::de::Error;
use self::serde::de::{Deserialize, Deserializer};


use self::serde_json::Map;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource(pub Map<String, Value>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyword {
    #[serde(rename="type")]
    pub t: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub ns: Option<String>,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub key: Keyword,
    pub val: Pattern,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Pattern(Pattern),
    ComplexValue {
        traits: Option<Vec<Member>>,
        val: Option<Pattern>,
        def: Option<i8>,
    },
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match self {
            &Value::Pattern(ref v) => Pattern::serialize(v, serializer),
            &Value::ComplexValue { ref val, ref traits, ref def } => {
                let mut num_fields = 1;
                if !val.is_none() {
                    num_fields += 1
                };
                if !def.is_none() {
                    num_fields += 1
                };
                let mut map = serializer.serialize_map(Some(num_fields)).unwrap();
                if let &Some(ref v) = val {
                    try!(serializer.serialize_map_key(&mut map, "val"));
                    try!(serializer.serialize_map_value(&mut map, &v));
                }
                try!(serializer.serialize_map_key(&mut map, "traits"));
                try!(serializer.serialize_map_value(&mut map, traits));
                if let &Some(ref d) = def {
                    try!(serializer.serialize_map_key(&mut map, "def"));
                    try!(serializer.serialize_map_value(&mut map, d));
                }
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
                Ok(Value::Pattern(Pattern::Simple(String::from(value))))
            }

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Value, V::Error>
                where V: MapVisitor
            {
                let mut val: Option<Pattern> = None;
                let mut traits: Option<Vec<Member>> = None;
                let mut def: Option<i8> = None;
                while let Some(key) = try!(visitor.visit_key()) {
                    let key: String = key;
                    match &key as &str {
                        "val" => {
                            let value: String = try!(visitor.visit_value());
                            val = Some(Pattern::Simple(value));
                        }
                        "traits" => {
                            let value: Vec<Member> = try!(visitor.visit_value());
                            traits = Some(value);
                        }
                        "def" => {
                            let value: i8 = try!(visitor.visit_value());
                            def = Some(value);
                        }
                        _ => {}
                    }
                }
                try!(visitor.end());
                Ok(Value::ComplexValue {
                    val: val,
                    traits: traits,
                    def: def,
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PatternElement {
    String, //  Placeable
}

#[derive(Debug, PartialEq)]
pub enum Pattern {
    Simple(String),
    Complex(Vec<PatternElement>),
}

impl Serialize for Pattern {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match self {
            &Pattern::Simple(ref v) => serializer.serialize_str(v),
            &Pattern::Complex(ref v) => panic!(),
        }
    }
}

impl Deserialize for Pattern {
    fn deserialize<D>(deserializer: &mut D) -> Result<Pattern, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Pattern;

            fn visit_str<E>(&mut self, value: &str) -> Result<Pattern, E>
                where E: Error
            {
                Ok(Pattern::Simple(String::from(value)))
            }

            fn visit_seq<V>(&mut self, mut visitor: V) -> Result<Pattern, V::Error>
                where V: SeqVisitor
            {
                let mut content: Vec<PatternElement> = vec![];

                while let Some(elem) = try!(visitor.visit()) {
                    let elem: String = elem;
                }
                try!(visitor.end());
                Ok(Pattern::Complex(content))
            }
        }
        deserializer.deserialize_struct_field(FieldVisitor)
    }
}
