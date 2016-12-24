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
pub enum MemberKey {
    Keyword(Keyword),
    Number(String)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub key: MemberKey,
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

#[derive(Debug, PartialEq, Deserialize)]
pub enum Expression {
    ExternalArgument(String),
    EntityReference(String),
    Number(String),
    CallExpression {
        name: Box<Expression>,
        args: Vec<Expression>
    },
    SelectExpression {
        exp: Box<Expression>,
        vars: Vec<Member>,
        def: Option<i8>
    },
    KeyValueArgument {
        name: String,
        val: Box<Expression>
    },
    Member {
        obj: Box<Expression>,
        key: MemberKey,
    },
    Pattern(String),
}

impl Serialize for Expression {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match self {
            &Expression::ExternalArgument(ref name) => {
                let mut map = serializer.serialize_map(Some(2)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "ext"));
                try!(serializer.serialize_map_key(&mut map, "name"));
                try!(serializer.serialize_map_value(&mut map, name));
                serializer.serialize_map_end(map)
            },
            &Expression::EntityReference(ref name) => {
                let mut map = serializer.serialize_map(Some(2)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "ref"));
                try!(serializer.serialize_map_key(&mut map, "name"));
                try!(serializer.serialize_map_value(&mut map, name));
                serializer.serialize_map_end(map)
            },
            &Expression::Number(ref val) => {
                let mut map = serializer.serialize_map(Some(2)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "num"));
                try!(serializer.serialize_map_key(&mut map, "val"));
                try!(serializer.serialize_map_value(&mut map, val));
                serializer.serialize_map_end(map)
            },
            &Expression::SelectExpression { ref exp, ref vars, ..} => {
                let mut map = serializer.serialize_map(Some(3)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "sel"));
                try!(serializer.serialize_map_key(&mut map, "exp"));
                try!(serializer.serialize_map_value(&mut map, exp));
                try!(serializer.serialize_map_key(&mut map, "vars"));
                try!(serializer.serialize_map_value(&mut map, vars));
                serializer.serialize_map_end(map)
            },
            &Expression::CallExpression { ref name, ref args} => {
                let mut map = serializer.serialize_map(Some(3)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "call"));
                try!(serializer.serialize_map_key(&mut map, "name"));
                try!(serializer.serialize_map_value(&mut map, name));
                try!(serializer.serialize_map_key(&mut map, "args"));
                try!(serializer.serialize_map_value(&mut map, args));
                serializer.serialize_map_end(map)
            },
            &Expression::KeyValueArgument { ref name, ref val} => {
                let mut map = serializer.serialize_map(Some(3)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "kw"));
                try!(serializer.serialize_map_key(&mut map, "name"));
                try!(serializer.serialize_map_value(&mut map, name));
                try!(serializer.serialize_map_key(&mut map, "val"));
                try!(serializer.serialize_map_value(&mut map, val));
                serializer.serialize_map_end(map)
            },
            &Expression::Member { ref obj, ref key} => {
                let mut map = serializer.serialize_map(Some(3)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "mem"));
                try!(serializer.serialize_map_key(&mut map, "obj"));
                try!(serializer.serialize_map_value(&mut map, obj));
                try!(serializer.serialize_map_key(&mut map, "key"));
                try!(serializer.serialize_map_value(&mut map, key));
                serializer.serialize_map_end(map)
            },
            &Expression::Pattern(ref val) => {
                serializer.serialize_str(val)
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum PatternElement {
    TextElement(String),
    PlaceableElement(Vec<Expression>),
}

impl Serialize for PatternElement {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match self {
            &PatternElement::TextElement(ref v) => serializer.serialize_str(v),
            &PatternElement::PlaceableElement(ref v) => {
                let mut state = try!(serializer.serialize_seq(Some(v.len())));
                for e in v {
                  try!(serializer.serialize_seq_elt(&mut state, e));
                }
                serializer.serialize_seq_end(state)
            }

        }
    }
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
            &Pattern::Complex(ref v) => {
                let mut state = try!(serializer.serialize_seq(Some(v.len())));
                for e in v {
                  try!(serializer.serialize_seq_elt(&mut state, e));
                }
                serializer.serialize_seq_end(state)
            }

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
