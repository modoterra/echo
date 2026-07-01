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
pub extern "C" fn echo_php_array_unique(array: EchoValue, flags: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(flags) = flags.php_int_value() else {
        return EchoValue::error();
    };
    if flags != 2 {
        return EchoValue::error();
    }
    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut seen: Vec<Vec<u8>> = Vec::new();
    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        let Some(string_key) = value.string_bytes() else {
            return EchoValue::error();
        };
        if seen.iter().any(|existing| existing == &string_key) {
            continue;
        }
        seen.push(string_key);
        keys.push(key.clone());
        values.push(*value);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_diff(array: EchoValue, other: EchoValue) -> EchoValue {
    if !array.is_array() || !other.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(other) = (unsafe { (other.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut other_values = Vec::with_capacity(other.values.len());
    for value in &other.values {
        let Some(bytes) = value.string_bytes() else {
            return EchoValue::error();
        };
        other_values.push(bytes);
    }

    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        let Some(bytes) = value.string_bytes() else {
            return EchoValue::error();
        };
        if !other_values.iter().any(|other| other == &bytes) {
            keys.push(key.clone());
            values.push(*value);
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_intersect(array: EchoValue, other: EchoValue) -> EchoValue {
    if !array.is_array() || !other.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(other) = (unsafe { (other.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut other_values = Vec::with_capacity(other.values.len());
    for value in &other.values {
        let Some(bytes) = value.string_bytes() else {
            return EchoValue::error();
        };
        other_values.push(bytes);
    }

    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        let Some(bytes) = value.string_bytes() else {
            return EchoValue::error();
        };
        if other_values.iter().any(|other| other == &bytes) {
            keys.push(key.clone());
            values.push(*value);
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
