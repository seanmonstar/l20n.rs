/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

#![crate_name = "l20n"]
#![license = "MPLv2"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]
#![doc(html_logo_url = "http://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "http://www.rust-lang.org/favicon.ico",
       html_root_url = "http://static.rust-lang.org/doc/master")]
// FIXME: rust#14450: add deny(warnings)
#![deny(missing_doc)]

extern crate serialize;

pub use context::{Locale, LocalizeResult, LocalizeError};
pub use data::{Data, Decoder, Encoder, DecodeError, EncodeError};
pub use compiler::ResolveError;
pub use parser::{ParseError, ParseErrorKind};

mod compiler;
mod data;
mod context;
mod parser;
