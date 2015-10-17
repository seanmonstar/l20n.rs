//! L20n implementation for localization
//!
//! This crate provides an interface to use l20n files to localize your
//! application.
//!
//! # Example usage
//!
//! ```rust
//! extern crate serialize;
//! extern crate l20n;
//!
//! use l20n::Locale;
//!
//! #[deriving(Encodable)]
//! struct Env {
//!   name: &'static str
//! }
//!
//! #[deriving(Decodable)]
//! struct Strings {
//!   hi: String
//! }
//!
//! fn main() {
//!   let mut locale = Locale::new();
//!   locale.add_resource(r#"
//!   <hi "Hello {{ $name }}!">
//!   "#).unwrap();
//!
//!   let env = Env { name: "Rust" };
//!   let strs: Strings = locale.localize_data(env).unwrap();
//!   assert_eq!(strs.hi.as_slice(), "Hello Rust!");
//! }
//! ```

#![deny(missing_docs)]
//#![cfg_attr(test, deny(warnings))]

extern crate serde;

pub use context::{Locale, LocalizeResult, LocalizeError};
pub use data::{DecodeError, EncodeError};
pub use compiler::ResolveError;
pub use parser::{ParseError, ParseErrorKind};

mod compiler;
mod data;
mod context;
mod parser;
