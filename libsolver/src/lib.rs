#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;
extern crate capnp;
extern crate futures;
extern crate futures_cpupool;

// These mods must be public in order to be exported.
pub mod ffi;
pub mod http;
pub mod capnproto;

mod error;
mod solver;
mod context;

pub mod api_capnp {
    include!(concat!(env!("OUT_DIR"), "/api_capnp.rs"));
}
