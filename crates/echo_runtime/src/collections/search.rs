use crate::{ECHO_VALUE_INT, ECHO_VALUE_STRING, EchoValue, echo_values_equal, php_values_equal};

use super::{EchoArray, EchoArrayKey};

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_count_values(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut keys: Vec<EchoArrayKey> = Vec::new();
    let mut values: Vec<EchoValue> = Vec::new();
    for value in &array.values {
        if !matches!(value.kind, ECHO_VALUE_INT | ECHO_VALUE_STRING) {
            continue;
        }
        let Some(key) = EchoArrayKey::from_value(*value) else {
            continue;
        };
        match keys.iter().position(|existing| existing == &key) {
            Some(index) => values[index] = EchoValue::int(values[index].payload as i64 + 1),
            None => {
                keys.push(key);
                values.push(EchoValue::int(1));
            }
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_in_array(
    needle: EchoValue,
    haystack: EchoValue,
    strict: EchoValue,
) -> EchoValue {
    if !haystack.is_array() {
        return EchoValue::error();
    }

    let Some(haystack) = (unsafe { (haystack.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let strict = strict.bool_value().unwrap_or(false);

    EchoValue::bool(haystack.values.iter().any(|value| {
        if strict {
            echo_values_equal(needle, *value)
        } else {
            php_values_equal(needle, *value)
        }
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_search(
    needle: EchoValue,
    haystack: EchoValue,
    strict: EchoValue,
) -> EchoValue {
    if !haystack.is_array() {
        return EchoValue::error();
    }

    let Some(haystack) = (unsafe { (haystack.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let strict = strict.bool_value().unwrap_or(false);

    for (key, value) in haystack.keys.iter().zip(&haystack.values) {
        let matches = if strict {
            echo_values_equal(needle, *value)
        } else {
            php_values_equal(needle, *value)
        };
        if matches {
            return key.to_value();
        }
    }

    EchoValue::bool(false)
}
