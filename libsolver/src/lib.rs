#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

// This mod must be public in order to be exported.
pub mod ffi;

mod error;
mod solver;
mod http;
