use std::collections::HashMap;

use serde;

pub use self::Data::*;

/// An internal Data format used to resolve L20n resources.
#[doc(hidden)]
#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    Null,
    Bool(bool),
    Num(i64),
    Str(String),
    List(Vec<Data>),
    Map(HashMap<String, Data>),
}

impl Data {
    pub fn get(&self, key: &str) -> Option<&Data> {
        match *self {
            Data::Map(ref map) => map.get(key),
            _ => None
        }
    }
}

#[doc(hidden)]
pub struct Encoder {
    data: Vec<Data>
}

impl Encoder {
    #[doc(hidden)]
    pub fn new() -> Encoder {
        Encoder { data: vec![] }
    }

    #[doc(hidden)]
    pub fn data(mut self) -> Option<Data> {
        self.data.pop()
    }
}

/// Errors that occur encoding environment data into something the L20n
/// resources can use.
#[derive(Debug)]
pub enum EncodeError {
    /// Type is not usable in L20n.
    UnsupportedType,
    /// Maps in L20n require keys to be Strings.
    KeyIsNotString,
    /// A map element is missing.
    MissingElements,
}

pub type EncoderResult = Result<(), EncodeError>;


impl serde::Serializer for Encoder {
    type Error = EncodeError;

    fn visit_bool(&mut self, v: bool) -> EncoderResult { self.data.push(Bool(v)); Ok(()) }
    fn visit_i64(&mut self, v: i64) -> EncoderResult { self.data.push(Num(v)); Ok(()) }
    fn visit_u64(&mut self, v: u64) -> EncoderResult { self.visit_i64(v as i64) }
    fn visit_f64(&mut self, v: f64) -> EncoderResult { self.visit_i64(v as i64) }

    fn visit_str(&mut self, v: &str) -> EncoderResult {
        self.data.push(Str(v.to_string()));
        Ok(())
    }

    fn visit_unit(&mut self) -> EncoderResult { self.data.push(Null); Ok(()) }
    fn visit_none(&mut self) -> EncoderResult { self.visit_unit() }
    fn visit_some<V>(&mut self, v: V) -> EncoderResult where V: serde::Serialize { v.serialize(self) }
    fn visit_seq<V>(&mut self, mut v: V) -> EncoderResult where V: serde::ser::SeqVisitor {
        self.data.push(List(vec![]));
        v.visit(self).map(|_| ())
    }
    fn visit_seq_elt<V>(&mut self, v: V) -> EncoderResult where V: serde::Serialize {
        match self.data.pop() {
            Some(List(mut list)) => {
                let mut elt_encoder = Encoder::new();
                try!(v.serialize(&mut elt_encoder));
                list.push(elt_encoder.data.pop().unwrap());
                self.data.push(List(list));
                Ok(())
            },
            _ => Err(EncodeError::UnsupportedType)
        }
    }
    fn visit_map<V>(&mut self, mut v: V) -> EncoderResult where V: serde::ser::MapVisitor {
        self.data.push(Map(HashMap::new()));
        v.visit(self).map(|_| ())
    }
    fn visit_map_elt<K, V>(&mut self, k: K, v: V) -> EncoderResult where K: serde::Serialize, V: serde::Serialize {
        match self.data.pop() {
            Some(Map(mut map)) => {
                let mut map_encoder = Encoder::new();
                try!(k.serialize(&mut map_encoder));
                let k = match map_encoder.data.pop() {
                    Some(Data::Str(s)) => s,
                    _ => return Err(EncodeError::KeyIsNotString)
                };

                try!(v.serialize(&mut map_encoder));
                let v = map_encoder.data.pop().unwrap();
                map.insert(k, v);
                self.data.push(Map(map));
                Ok(())
            },
            _ => Err(EncodeError::UnsupportedType)
        }
    }
}

pub struct Decoder {
    data: Vec<Data>
}

impl Decoder {
    /// Creates a new Decoder.
    pub fn new(data: Data) -> Decoder {
        Decoder {
            data: vec![data]
        }
    }
}

impl serde::Deserializer for Decoder {
    type Error = serde::de::value::Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor {
        match self.data.pop() {
            Some(Data::Null) => visitor.visit_unit(),
            Some(Data::Bool(b)) => visitor.visit_bool(b),
            Some(Data::Num(n)) => visitor.visit_i64(n),
            Some(Data::Str(s)) => visitor.visit_str(&s),
            Some(Data::List(list)) => {
                let len = list.len();
                visitor.visit_seq(serde::de::value::SeqDeserializer::new(list.into_iter(), len))
            }
            Some(Data::Map(map)) => {
                let len = map.len();
                visitor.visit_map(serde::de::value::MapDeserializer::new(map.into_iter(), len))
            },
            None => unreachable!()
        }
    }
}

impl serde::de::value::ValueDeserializer for Data {
    type Deserializer = Decoder;
    fn into_deserializer(self) -> Decoder {
        Decoder::new(self)
    }
}

