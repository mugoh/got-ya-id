#![recursion_limit = "128"]
pub mod apps;
pub mod config;
pub mod core;
pub mod diesel_cfg;
pub mod errors;

#[macro_use(lazy_static)]
extern crate lazy_static;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate log;

extern crate reqwest;
