pub use crate::{EchoArray, EchoCallable, EchoList, EchoSymbol, EchoValue};

pub(crate) fn format_php_float(value: f64) -> String {
    if value.is_nan() {
        return "NAN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-INF".to_string()
        } else {
            "INF".to_string()
        };
    }

    let formatted = format!("{value:.14}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[derive(Debug)]
pub struct EchoString {
    pub(crate) bytes: Vec<u8>,
}

impl EchoString {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

#[derive(Debug)]
pub struct EchoObject {
    pub(crate) fields: Vec<(String, EchoValue)>,
}

impl EchoObject {
    pub(crate) fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_float_formatting_matches_runtime_scalar_strings() {
        assert_eq!(format_php_float(12.5), "12.5");
        assert_eq!(format_php_float(12.0), "12");
        assert_eq!(format_php_float(f64::INFINITY), "INF");
        assert_eq!(format_php_float(f64::NEG_INFINITY), "-INF");
        assert_eq!(format_php_float(f64::NAN), "NAN");
    }
}
