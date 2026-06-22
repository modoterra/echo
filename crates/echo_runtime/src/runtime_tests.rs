use super::*;

fn test_string_value(bytes: &[u8]) -> EchoValue {
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes.to_vec()))))
}

fn assert_float_value(value: EchoValue, expected: f64) {
    assert_eq!(value.kind, ECHO_VALUE_FLOAT);
    assert!((f64::from_bits(value.payload) - expected).abs() < 0.000000000001);
}

mod arithmetic;
mod callable;
mod collections;
mod encoding;
mod environment;
mod filesystem;
mod math;
mod scalar;
mod stdlib;
mod string;
mod string_collections;
mod string_compare;
mod string_format;
mod string_paths;
mod string_rewrite;
mod string_search;
mod string_span;
mod task;
mod value;
