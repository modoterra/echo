#![recursion_limit = "256"]

mod grammar;

pub use grammar::{parse, parse_source_file, parse_trusted_std, parse_trusted_std_source};
