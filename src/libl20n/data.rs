/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::str;

use serialize;

/// An internal Data format used to resolve L20n resources.
#[doc(hidden)]
#[deriving(Show, PartialEq, Clone)]
pub enum Data {
  Null,
  Bool(bool),
  Num(int),
  Str(String),
  List(Vec<Data>),
  Map(HashMap<String, Data>),
}

impl Data {

  /// Convenience method to unwrapping a Map and calling the same method
  /// on it.
  #[doc(hidden)]
  pub fn find_copy(&self, key: &String) -> Option<Data> {
    match *self {
      Map(ref map) => map.find_copy(key),
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
#[deriving(Show)]
pub enum EncodeError {
  /// Type is not usable in L20n.
  UnsupportedType,
  /// Maps in L20n require keys to be Strings.
  KeyIsNotString,
  /// A map element is missing.
  MissingElements,
}

pub type EncoderResult = Result<(), EncodeError>;


impl<'a> serialize::Encoder<EncodeError> for Encoder {
    fn emit_nil(&mut self) -> EncoderResult { self.data.push(Null); Ok(()) }

    fn emit_uint(&mut self, v: uint) -> EncoderResult { self.emit_int(v as int) }
    fn emit_u64(&mut self, v: u64) -> EncoderResult { self.emit_int(v as int) }
    fn emit_u32(&mut self, v: u32) -> EncoderResult { self.emit_int(v as int) }
    fn emit_u16(&mut self, v: u16) -> EncoderResult { self.emit_int(v as int) }
    fn emit_u8(&mut self, v: u8) -> EncoderResult { self.emit_int(v as int) }

    fn emit_int(&mut self, v: int) -> EncoderResult { self.data.push(Num(v)); Ok(()) }
    fn emit_i64(&mut self, v: i64) -> EncoderResult { self.emit_int(v as int) }
    fn emit_i32(&mut self, v: i32) -> EncoderResult { self.emit_int(v as int) }
    fn emit_i16(&mut self, v: i16) -> EncoderResult { self.emit_int(v as int) }
    fn emit_i8(&mut self, v: i8) -> EncoderResult { self.emit_int(v as int) }

    fn emit_bool(&mut self, v: bool) -> EncoderResult { self.data.push(Bool(v)); Ok(()) }

    fn emit_f64(&mut self, v: f64) -> EncoderResult {
        self.emit_int(v as int)
    }

    fn emit_f32(&mut self, v: f32) -> EncoderResult {
        self.emit_int(v as int)
    }

    fn emit_char(&mut self, v: char) -> EncoderResult {
        self.data.push(Str(str::from_char(v)));
        Ok(())
    }

    fn emit_str(&mut self, v: &str) -> EncoderResult {
        self.data.push(Str(v.to_string()));
        Ok(())
    }

    fn emit_enum(&mut self, _name: &str, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
      f(self)
    }

    fn emit_enum_variant(&mut self,
                         name: &str,
                         _id: uint,
                         len: uint,
                         _f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
      if len == 0 {
        self.emit_str(name)
      } else {
        Err(UnsupportedType)
      }
    }

    fn emit_enum_variant_arg(&mut self,
                             _a_idx: uint,
                             _f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        Err(UnsupportedType)
    }

    fn emit_enum_struct_variant(&mut self,
                                _v_name: &str,
                                _v_id: uint,
                                _len: uint,
                                _f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        Err(UnsupportedType)
    }

    fn emit_enum_struct_variant_field(&mut self,
                                      _f_name: &str,
                                      _f_idx: uint,
                                      _f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        Err(UnsupportedType)
    }

    fn emit_struct(&mut self,
                   _name: &str,
                   _len: uint,
                   f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        self.data.push(Map(HashMap::new()));
        f(self)
    }

    fn emit_struct_field(&mut self,
                         name: &str,
                         _idx: uint,
                         f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        let mut m = match self.data.pop() {
            Some(Map(m)) => m,
            _ => { return Err(UnsupportedType); }
        };
        try!(f(self));
        let data = match self.data.pop() {
            Some(d) => d,
            _ => { return Err(UnsupportedType); }
        };
        m.insert(name.to_string(), data);
        self.data.push(Map(m));
        Ok(())
    }

    fn emit_tuple(&mut self,
                  len: uint,
                  f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        self.emit_seq(len, f)
    }

    fn emit_tuple_arg(&mut self, idx: uint, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct(&mut self,
                         _name: &str,
                         len: uint,
                         f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        self.emit_seq(len, f)
    }

    fn emit_tuple_struct_arg(&mut self, idx: uint, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        self.emit_seq_elt(idx, f)
    }

    // Specialized types:
    fn emit_option(&mut self, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
      f(self)
    }

    fn emit_option_none(&mut self) -> EncoderResult {
      self.emit_nil()
    }

    fn emit_option_some(&mut self, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
      f(self)
    }

    fn emit_seq(&mut self, _len: uint, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        self.data.push(List(Vec::new()));
        f(self)
    }

    fn emit_seq_elt(&mut self, _idx: uint, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        let mut v = match self.data.pop() {
            Some(List(v)) => v,
            _ => { return Err(UnsupportedType); }
        };
        try!(f(self));
        let data = match self.data.pop() {
            Some(d) => d,
            _ => { return Err(UnsupportedType); }
        };
        v.push(data);
        self.data.push(List(v));
        Ok(())
    }

    fn emit_map(&mut self, _len: uint, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        self.data.push(Map(HashMap::new()));
        f(self)
    }

    fn emit_map_elt_key(&mut self, _idx: uint, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        try!(f(self));
        let last = match self.data.last() {
            Some(d) => d,
            None => { return Err(MissingElements); }
        };
        match *last {
            Str(_) => Ok(()),
            _ => Err(KeyIsNotString),
        }
    }

    fn emit_map_elt_val(&mut self, _idx: uint, f: |&mut Encoder| -> EncoderResult) -> EncoderResult {
        let k = match self.data.pop() {
            Some(Str(s)) => s,
            _ => { return Err(KeyIsNotString); }
        };
        let mut m = match self.data.pop() {
            Some(Map(m)) => m,
            _ => fail!("Expected a map"),
        };
        try!(f(self));
        let popped = match self.data.pop() {
            Some(p) => p,
            None => fail!("Error: Nothing to pop!"),
        };
        m.insert(k, popped);
        self.data.push(Map(m));
        Ok(())
    }
}

#[doc(hidden)]
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

pub type DecodeResult<T> = Result<T, DecodeError>;

/// Errors that can occur decoding an L20n file into a Rust value.
#[deriving(Show)]
pub enum DecodeError {
  /// The type being requested doesn't match what the L20n file outputs.
  WrongType,
  /// A string was request from L20n that wasn't in the resources.
  MissingField(String)
}

impl serialize::Decoder<DecodeError> for Decoder {
    fn read_nil(&mut self) -> DecodeResult<()> {
        match self.data.pop() {
          Some(Null) => Ok(()),
          _ => Err(WrongType)
        }
    }

    fn read_u64(&mut self) -> DecodeResult<u64 > { Ok(try!(self.read_int()) as u64) }
    fn read_u32(&mut self) -> DecodeResult<u32 > { Ok(try!(self.read_int()) as u32) }
    fn read_u16(&mut self) -> DecodeResult<u16 > { Ok(try!(self.read_int()) as u16) }
    fn read_u8 (&mut self) -> DecodeResult<u8 > { Ok(try!(self.read_int()) as u8) }
    fn read_uint(&mut self) -> DecodeResult<uint> { Ok(try!(self.read_int()) as uint) }

    fn read_i64(&mut self) -> DecodeResult<i64> { Ok(try!(self.read_int()) as i64) }
    fn read_i32(&mut self) -> DecodeResult<i32> { Ok(try!(self.read_int()) as i32) }
    fn read_i16(&mut self) -> DecodeResult<i16> { Ok(try!(self.read_int()) as i16) }
    fn read_i8 (&mut self) -> DecodeResult<i8 > { Ok(try!(self.read_int()) as i8) }
    fn read_int(&mut self) -> DecodeResult<int> { Ok(try!(self.read_int()) as int) }

    fn read_bool(&mut self) -> DecodeResult<bool> {
      match self.data.pop() {
        Some(Bool(b)) => Ok(b),
        _ => Err(WrongType)
      }
    }

    fn read_f64(&mut self) -> DecodeResult<f64> {
      match self.data.pop() {
        Some(Num(f)) => Ok(f as f64),
        _ => Err(WrongType)
      }
    }

    fn read_f32(&mut self) -> DecodeResult<f32> { Ok(try!(self.read_int()) as f32) }

    fn read_char(&mut self) -> DecodeResult<char> {
        let s = try!(self.read_str());
        {
            let mut it = s.as_slice().chars();
            match (it.next(), it.next()) {
                // exactly one character
                (Some(c), None) => return Ok(c),
                _ => ()
            }
        }
        Err(WrongType)
    }

    fn read_str(&mut self) -> DecodeResult<String> {
        match self.data.pop() {
          Some(Str(s)) => Ok(s),
          _ => Err(WrongType)
        }
    }

    fn read_enum<T>(&mut self,
                    _name: &str,
                    f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        
        f(self)
    }

    fn read_enum_variant<T>(&mut self,
                            _names: &[&str],
                            _f: |&mut Decoder, uint| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        Err(WrongType)
    }

    fn read_enum_variant_arg<T>(&mut self, _idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                                -> DecodeResult<T> {
        
        f(self)
    }

    fn read_enum_struct_variant<T>(&mut self,
                                   names: &[&str],
                                   f: |&mut Decoder, uint| -> DecodeResult<T>)
                                   -> DecodeResult<T> {
        
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T>(&mut self,
                                         _name: &str,
                                         idx: uint,
                                         f: |&mut Decoder| -> DecodeResult<T>)
                                         -> DecodeResult<T> {
        
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T>(&mut self,
                      _name: &str,
                      _len: uint,
                      f: |&mut Decoder| -> DecodeResult<T>)
                      -> DecodeResult<T> {
        
        let value = try!(f(self));
        self.data.pop();
        Ok(value)
    }

    fn read_struct_field<T>(&mut self,
                            name: &str,
                            _idx: uint,
                            f: |&mut Decoder| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        
        let mut obj = match self.data.pop() {
          Some(Map(m)) => m,
          _ => return Err(WrongType)
        };

        let value = match obj.pop(&name.to_string()) {
            None => return Err(MissingField(name.to_string())),
            Some(data) => {
                self.data.push(data);
                try!(f(self))
            }
        };
        self.data.push(Map(obj));
        Ok(value)
    }

    fn read_tuple<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        
        self.read_seq(f)
    }

    fn read_tuple_arg<T>(&mut self,
                         idx: uint,
                         f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T>(&mut self,
                            _name: &str,
                            f: |&mut Decoder, uint| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        
        self.read_tuple(f)
    }

    fn read_tuple_struct_arg<T>(&mut self,
                                idx: uint,
                                f: |&mut Decoder| -> DecodeResult<T>)
                                -> DecodeResult<T> {
        
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T>(&mut self, f: |&mut Decoder, bool| -> DecodeResult<T>) -> DecodeResult<T> {
        match self.data.pop() {
            Some(Null) => f(self, false),
            Some(value) => { self.data.push(value); f(self, true) }
            _ => Err(WrongType)
        }
    }

    fn read_seq<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        
        let list = match self.data.pop() {
          Some(List(list)) => list,
          _ => return Err(WrongType)
        };
        let len = list.len();
        for v in list.move_iter().rev() {
            self.data.push(v);
        }
        f(self, len)
    }

    fn read_seq_elt<T>(&mut self,
                       _idx: uint,
                       f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        
        f(self)
    }

    fn read_map<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        let obj = match self.data.pop() {
          Some(Map(m)) => m,
          _ => return Err(WrongType)
        };
        let len = obj.len();
        for (key, value) in obj.move_iter() {
            self.data.push(value);
            self.data.push(Str(key));
        }
        f(self, len)
    }

    fn read_map_elt_key<T>(&mut self, _idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                           -> DecodeResult<T> {
        f(self)
    }

    fn read_map_elt_val<T>(&mut self, _idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                           -> DecodeResult<T> {
        f(self)
    }
}
