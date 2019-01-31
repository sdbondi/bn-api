#![deny(unreachable_patterns)]
#![deny(unknown_lints)]
#![cfg_attr(not(debug_assertions), deny(unused_variables))]
#![cfg_attr(not(debug_assertions), deny(unused_imports))]
#![cfg_attr(not(debug_assertions), deny(dead_code))]
// Unused results is more often than not an error
#![deny(unused_must_use)]
extern crate actix_web;
extern crate ring;
extern crate serde;

pub mod caching;
