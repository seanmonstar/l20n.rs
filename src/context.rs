use std::collections::HashMap;

use serialize;

use compiler::{Resolve, ResolveContext};
use compiler;
use data;
use parser;


pub struct Context {
  locales: HashMap<String, Locale>,
  default_locale: String
}

impl Context {
  pub fn new() -> Context {
    Context::with_default(String::from_str("i-default"))
  }

  pub fn with_default(locale: String) -> Context {
    let mut locales = HashMap::new();
    locales.insert(locale.clone(), Locale::new());
    Context {
      locales: locales,
      default_locale: locale
    }
  }


  /*
  pub fn add_resource(&mut self, res: String) -> Result<(), parser::ParseError>{
    self.add_locale_resource("i-default".to_string(), res) //self.default_locale.clone(), res)
  }


  pub fn add_locale_resource(&mut self, name: String, res: String) -> Result<(), parser::ParseError> {
    let mut locale = self.locales.find_or_insert_with(name, |_| Locale::new());
    let entities = try!(compiler::compile(res.as_slice()));
    locale.resources.extend(entities.move_iter());
    Ok(())
  }

  pub fn locale<'a>(&'a self) -> &'a Locale {
    self.get_locale(self.default_locale.as_slice()).unwrap()
  }

  pub fn get_locale<'a>(&'a self, name: &str) -> Option<&'a Locale> {
    self.locales.find_equiv(&name)
  }
  */

}


/// A Locale contains all the resources for a specific language.
pub struct Locale {
  resources: HashMap<String, parser::Entry>
}

/// An enum of the various errors that can occur during localization.
#[deriving(Show)]
pub enum LocalizeError {
  /// Wraps a DecodeError.
  DecodeError(data::DecodeError),
  /// Wraps an EncodeError.
  EncodeError(data::EncodeError),
  /// Wraps a ResolveError.
  ResolveError(compiler::ResolveError)
}

pub type LocalizeResult<T> = Result<T, LocalizeError>;

impl Locale {

  /// Creates a new empty Locale.
  pub fn new() -> Locale {
    Locale {
      resources: HashMap::new()
    }
  }

  /// Add a L20n string resource, and it will be parsed.
  pub fn add_resource(&mut self, res: &str) -> Result<(), parser::ParseError> {
    let entities = try!(compiler::compile(res));
    self.resources.extend(entities.into_iter());
    Ok(())
  }

  /// Resolves all the resouces into Strings, and returns a Decodable
  /// object of your choosing.
  pub fn localize<T: serialize::Decodable<data::Decoder, data::DecodeError>>(&self) -> LocalizeResult<T> {
    self.localize_data_raw(data::Null)
  }

  /// Same as `localize`, but you provide environment Data for the L20n
  /// files to use.
  pub fn localize_data<
    T: serialize::Decodable<data::Decoder, data::DecodeError>,
    D: serialize::Encodable<data::Encoder, data::EncodeError>
    >(&self, data: D) -> LocalizeResult<T> {
    let mut enc = data::Encoder::new();
    match data.encode(&mut enc) {
      Err(e) => return Err(EncodeError(e)),
      _ => {}
    }
    self.localize_data_raw(enc.data().unwrap())
  }

  fn localize_data_raw<
    T: serialize::Decodable<data::Decoder, data::DecodeError>
    >(&self, data: data::Data) -> LocalizeResult<T> {
    let mut map = HashMap::new();
    let ctx = ResolveContext::new(&self.resources, &data);
    for (id, entry) in self.resources.iter() {
      map.insert(id.clone(), match entry.resolve_data(&ctx) {
        Ok(d) => d,
        Err(e) => return Err(ResolveError(e))
      });
    }

    let mut dec = data::Decoder::new(data::Map(map));
    match serialize::Decodable::decode(&mut dec) {
      Err(e) => Err(DecodeError(e)),
      Ok(t) => Ok(t)
    }
  }
}

#[cfg(test)]
mod tests {

  use super::Locale;

  #[deriving(Decodable)]
  struct Translated {
    hi: String,
    factorial: String,
    mail: String,
  }

  #[deriving(Encodable)]
  struct Values {
    number: int
  }

  #[test]
  fn test_locale() {
    let mut locale = Locale::new();
    let src = r#"
    <brand 'Rust' long: 'Rust Lang'>
    <hi 'Hello, {{ brand::long }}!'>
    <many['zero'] { zero: 'none', one: 'one', many: 'too many' }>
    <mail 'Email in your inbox: {{ many.many }}.'>
    <fac($n) { $n == 0 ? 1 : $n * fac($n -1) }>
    <factorial "Factorial of {{ $number }} is {{ fac($number) }}.">
    "#;
    locale.add_resource(src).unwrap();

    let data = Values { number: 3 };
    let t: Translated = locale.localize_data(data).unwrap();

    assert_eq!(t.hi.as_slice(), "Hello, Rust Lang!");
    assert_eq!(t.factorial.as_slice(), "Factorial of 3 is 6.");
    assert_eq!(t.mail.as_slice(), "Email in your inbox: too many.");
  }

}
