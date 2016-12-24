extern crate serde;
extern crate serde_json;

use self::serde::ser::{Serialize, Serializer};
use self::serde::de::{Deserialize, Deserializer, Visitor, MapVisitor, SeqVisitor, Error};
use self::serde::de::value::{ValueDeserializer, SeqVisitorDeserializer};


use self::serde_json::Map;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource(pub Map<String, Value>);

#[derive(Debug, PartialEq, Deserialize)]
pub struct Keyword {
    #[serde(skip_serializing_if="Option::is_none")]
    pub ns: Option<String>,
    pub name: String,
}

impl Serialize for Keyword {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        let num_fields = if self.ns.is_some() { 3 } else { 2 };

        let mut map = serializer.serialize_map(Some(num_fields)).unwrap();
        try!(serializer.serialize_map_key(&mut map, "type"));
        try!(serializer.serialize_map_value(&mut map, "kw"));
        try!(serializer.serialize_map_key(&mut map, "name"));
        try!(serializer.serialize_map_value(&mut map, &self.name));
        match self.ns {
            Some(ref v) => {
                try!(serializer.serialize_map_key(&mut map, "ns"));
                try!(serializer.serialize_map_value(&mut map, v));
            },
            None => {}
        }
        serializer.serialize_map_end(map)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Number(pub String);

#[derive(Debug, PartialEq)]
pub enum MemberKey {
    Keyword(Keyword),
    Number(Number)
}

impl Serialize for MemberKey {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match self {
            &MemberKey::Keyword(ref v) => Keyword::serialize(v, serializer),
            &MemberKey::Number(ref v) => Number::serialize(v, serializer),
        }
    }
}

impl Deserialize for MemberKey {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = MemberKey;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<MemberKey, V::Error>
                where V: MapVisitor
            {
                let mut v: Option<String> = None;
                let mut t: Option<String> = None;
                let mut ns: Option<String> = None;

                while let Some(key) = try!(visitor.visit_key()) {
                    let key: String = key;
                    match &key as &str {
                        "name" => {
                            let val: String = try!(visitor.visit_value());
                            v = Some(val);
                        },
                        "type" => {
                            let val: String = try!(visitor.visit_value());
                            t = Some(val);
                        },
                        "ns" => {
                            let val: String = try!(visitor.visit_value());
                            ns = Some(val);
                        }
                        _ => {}
                    }
                }
                try!(visitor.end());

                match t {
                    Some(ch) => match ch.as_str() {
                        "kw" => Ok(MemberKey::Keyword(Keyword {
                            name: v.unwrap(),
                            ns: ns
                        })),
                        "num" => Ok(MemberKey::Number(Number(v.unwrap()))),
                        _ => panic!()
                    },
                    None => panic!()
                }
            }
        }

        deserializer.deserialize_struct_field(FieldVisitor)
    }
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
        match *self {
            Value::Pattern(ref v) => v.serialize(serializer),
            Value::ComplexValue { ref val, ref traits, ref def } => {
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Value;

            fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E>
                where E: Error
            {
                let mut deserializer = value.into_deserializer();
                Deserialize::deserialize(&mut deserializer).map(Value::Pattern)
            }


            fn visit_seq<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
                where V: SeqVisitor
            {
                let mut deserializer = SeqVisitorDeserializer::new(visitor);
                Deserialize::deserialize(&mut deserializer).map(Value::Pattern)
            }

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: MapVisitor
            {
                let mut val: Option<Pattern> = None;
                let mut traits: Option<Vec<Member>> = None;
                let mut def: Option<i8> = None;
                while let Some(key) = try!(visitor.visit_key()) {
                    let key: String = key;
                    match &key as &str {
                        "val" => {
                            val = Some(visitor.visit_value()?);
                        }
                        "traits" => {
                            traits = Some(visitor.visit_value()?);
                        }
                        "def" => {
                            def = Some(visitor.visit_value()?);
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

#[derive(Debug, PartialEq)]
pub enum Expression {
    ExternalArgument(String),
    EntityReference(String),
    Number(Number),
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
                Number::serialize(val, serializer)
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

impl Deserialize for Expression {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Expression;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
                where V: MapVisitor
            {
                let mut name: Option<String> = None;
                let mut t: Option<String> = None;

                while let Some(key) = visitor.visit_key::<String>()? {
                    if key == "type" {
                        t = Some(visitor.visit_value()?);
                    }
                    if key == "name" {
                        name = Some(visitor.visit_value()?);
                    }
                }
                visitor.end()?;

                let t = match t {
                    Some(t) => t,
                    None => visitor.missing_field("type")?,
                };

                let name = match name {
                    Some(name) => name,
                    None => visitor.missing_field("name")?,
                };

                match t.as_str() {
                    "ext" => {
                        Ok(Expression::ExternalArgument(name))
                    },
                    "ref" => {
                        Ok(Expression::EntityReference(name))
                    },
                    _ => {
                        panic!()
                    }
                }
            }
        }

        deserializer.deserialize_struct_field(FieldVisitor)
    }
}

#[derive(Debug, PartialEq)]
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

impl Deserialize for PatternElement {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = PatternElement;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(PatternElement::TextElement(v.to_owned()))
            }

            fn visit_seq<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
                where V: SeqVisitor
            {
                let mut deserializer = SeqVisitorDeserializer::new(visitor);
                Deserialize::deserialize(&mut deserializer).map(PatternElement::PlaceableElement)
            }
        }
        deserializer.deserialize_struct_field(FieldVisitor)
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
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = Pattern;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: Error
            {
                Ok(Pattern::Simple(v.to_owned()))
            }

            fn visit_seq<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
                where V: SeqVisitor
            {
                let mut deserializer = SeqVisitorDeserializer::new(visitor);
                Deserialize::deserialize(&mut deserializer).map(Pattern::Complex)
            }
        }
        deserializer.deserialize_struct_field(FieldVisitor)
    }
}
