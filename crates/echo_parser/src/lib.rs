#![recursion_limit = "256"]

mod grammar;

pub use grammar::{parse, parse_trusted_std};
