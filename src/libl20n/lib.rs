/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! L20n implementation for localization
//!
//! This crate provides an interface to use l20n files to localize your
//! application.

#![crate_id = "l20n#0.1-pre"]
#![license = "MPLv2"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![deny(missing_doc)]

extern crate serialize;

pub mod compiler;
pub mod data;
pub mod context;
pub mod parser;
