#![deprecated(note = "Look at https://github.com/projectfluent/fluent-rs instead.")]
//! L20n implementation for localization
//!
//! This crate provides an interface to use l20n files to localize your
//! application.
//!
//! # Example usage
//!
//! ```rust
//! extern crate l20n;
//!
//! use std::collections::HashMap;
//!
//! fn main() {
//!     let mut locale = l20n::Locale::new();
//!     locale.add_resource(r#"
//!     <hi "Hello {{ $name }}!">
//!     "#).unwrap();
//!
//!     let mut env = HashMap::new();
//!     env.insert("name", "Rust");
//!     let strs: HashMap<String, String> = locale.localize_data(env).unwrap();
//!     assert_eq!(strs["hi"], "Hello Rust!");
//! }
//! ```

#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

extern crate serde;

pub use context::{Locale, LocalizeResult, LocalizeError};
pub use data::{EncodeError};
pub use compiler::ResolveError;
pub use parser::{ParseError, ParseErrorKind};

mod compiler;
mod data;
mod context;
mod parser;
