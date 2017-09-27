
#[macro_use]
extern crate error_chain;

// This mod must be public in order to be exported.
pub mod ffi;

mod error;
mod solver;
