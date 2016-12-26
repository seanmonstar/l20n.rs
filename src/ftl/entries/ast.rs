extern crate serde;
extern crate serde_json;

use self::serde::ser::{Serialize, Serializer};
use self::serde::de::{Deserialize, Deserializer, Visitor, MapVisitor, SeqVisitor, Error};
use self::serde::de::value::{ValueDeserializer, SeqVisitorDeserializer, MapVisitorDeserializer};


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

#[derive(Debug, PartialEq, Deserialize)]
pub struct Number(pub String);

impl Serialize for Number {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        let mut map = serializer.serialize_map(Some(2)).unwrap();
        try!(serializer.serialize_map_key(&mut map, "type"));
        try!(serializer.serialize_map_value(&mut map, "num"));
        try!(serializer.serialize_map_key(&mut map, "val"));
        try!(serializer.serialize_map_value(&mut map, &self.0));
        serializer.serialize_map_end(map)
    }
}

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
            Value::Pattern(ref v) => match v {
                &Pattern::Simple(ref v) => v.serialize(serializer),
                &Pattern::Complex(ref v) => {
                    let mut map = serializer.serialize_map(Some(1)).unwrap();
                    try!(serializer.serialize_map_key(&mut map, "val"));
                    try!(serializer.serialize_map_value(&mut map, &v));
                    serializer.serialize_map_end(map)
                },
            },
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
    FunctionCall(String),
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
                try!(serializer.serialize_map_value(&mut map, "kv"));
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
            &Expression::FunctionCall(ref name) => {
                let mut map = serializer.serialize_map(Some(2)).unwrap();
                try!(serializer.serialize_map_key(&mut map, "type"));
                try!(serializer.serialize_map_value(&mut map, "fun"));
                try!(serializer.serialize_map_key(&mut map, "name"));
                try!(serializer.serialize_map_value(&mut map, name));
                serializer.serialize_map_end(map)
            },
            &Expression::Pattern(ref val) => {
                serializer.serialize_str(val)
            }
        }
    }
}

enum ExpressionName {
    String(String),
    Expression(Expression),
}

impl Deserialize for ExpressionName {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = ExpressionName;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: Error
            {
                let mut deserializer = v.into_deserializer();
                Deserialize::deserialize(&mut deserializer).map(ExpressionName::String)
            }

            fn visit_map<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
                where V: MapVisitor
            {
                let mut deserializer = MapVisitorDeserializer::new(visitor);
                Deserialize::deserialize(&mut deserializer).map(ExpressionName::Expression)
            }
        }
        deserializer.deserialize_struct_field(FieldVisitor)
    }
}

enum ExpressionValue {
    String(String),
    Expression(Expression),
}

impl Deserialize for ExpressionValue {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct FieldVisitor;

        impl Visitor for FieldVisitor {
            type Value = ExpressionValue;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: Error
            {
                let mut deserializer = v.into_deserializer();
                Deserialize::deserialize(&mut deserializer).map(ExpressionValue::String)
            }

            fn visit_map<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
                where V: MapVisitor
            {
                let mut deserializer = MapVisitorDeserializer::new(visitor);
                Deserialize::deserialize(&mut deserializer).map(ExpressionValue::Expression)
            }
        }
        deserializer.deserialize_struct_field(FieldVisitor)
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
                let mut name: Option<ExpressionName> = None;
                let mut t: Option<String> = None;
                let mut exp: Option<Expression> = None;
                let mut vars: Option<Vec<Member>> = None;
                let mut args: Option<Vec<Expression>> = None;
                let mut val: Option<ExpressionValue> = None;
                let mut obj: Option<Box<Expression>> = None;
                let mut k: Option<MemberKey> = None;

                while let Some(key) = visitor.visit_key::<String>()? {
                    match key.as_str() {
                        "type" => t = Some(visitor.visit_value()?),
                        "name" => name = Some(visitor.visit_value()?),
                        "exp" => exp = Some(visitor.visit_value()?),
                        "vars" => vars = Some(visitor.visit_value()?),
                        "args" => args = Some(visitor.visit_value()?),
                        "val" => val = Some(visitor.visit_value()?),
                        "obj" => obj = Some(visitor.visit_value()?),
                        "key" => k = Some(visitor.visit_value()?),
                        _ => {},
                    }
                }
                visitor.end()?;

                let t = match t {
                    Some(t) => t,
                    None => visitor.missing_field("type")?,
                };

                match t.as_str() {
                    "ext" => {
                        let name = match name {
                            Some(name) => match name {
                                ExpressionName::String(v) => v,
                                ExpressionName::Expression(_) => panic!(),
                            },
                            None => visitor.missing_field("name")?,
                        };
                        Ok(Expression::ExternalArgument(name))
                    },
                    "ref" => {
                        let name = match name {
                            Some(name) => match name {
                                ExpressionName::String(v) => v,
                                ExpressionName::Expression(_) => panic!(),
                            },
                            None => visitor.missing_field("name")?,
                        };
                        Ok(Expression::EntityReference(name))
                    },
                    "sel" => {
                        let exp = match exp {
                            Some(exp) => exp,
                            None => visitor.missing_field("exp")?,
                        };
                        let vars = match vars {
                            Some(vars) => vars,
                            None => visitor.missing_field("vars")?,
                        };
                        Ok(Expression::SelectExpression{
                            exp: Box::new(exp), 
                            vars: vars,
                            def: None
                        })
                    },
                    "call" => {
                        let name = match name {
                            Some(name) => match name {
                                ExpressionName::Expression(v) => v,
                                ExpressionName::String(_) => panic!(),
                            },
                            None => visitor.missing_field("name")?,
                        };
                        let args = match args {
                            Some(args) => args,
                            None => visitor.missing_field("args")?,
                        };
                        Ok(Expression::CallExpression{
                            name: Box::new(name), 
                            args: args,
                        })
                    },
                    "fun" => {
                        let name = match name {
                            Some(name) => match name {
                                ExpressionName::String(v) => v,
                                ExpressionName::Expression(_) => panic!(),
                            },
                            None => visitor.missing_field("name")?,
                        };
                        Ok(Expression::FunctionCall(name))
                    },
                    "num" => {
                        let val = match val {
                            Some(val) => match val {
                                ExpressionValue::String(v) => v,
                                ExpressionValue::Expression(_) => panic!(),
                            },
                            None => visitor.missing_field("val")?,
                        };
                        Ok(Expression::Number(Number(val)))
                    },
                    "kv" => {
                        let name = match name {
                            Some(name) => match name {
                                ExpressionName::String(v) => v,
                                ExpressionName::Expression(_) => panic!(),
                            },
                            None => visitor.missing_field("name")?,
                        };
                        let val = match val {
                            Some(val) => match val {
                                ExpressionValue::Expression(v) => v,
                                ExpressionValue::String(v) => {
                                    Expression::Pattern(v)
                                },
                            },
                            None => visitor.missing_field("val")?,
                        };
                        Ok(Expression::KeyValueArgument {
                            name: name,
                            val: Box::new(val)
                        })
                    },
                    "mem" => {
                        let obj = match obj {
                            Some(obj) => obj,
                            None => visitor.missing_field("obj")?,
                        };
                        let k = match k {
                            Some(k) => k,
                            None => visitor.missing_field("key")?,
                        };
                        Ok(Expression::Member {
                            obj: obj,
                            key: k
                        })
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
