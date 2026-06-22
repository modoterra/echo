use crate::{ECHO_VALUE_PENDING, EchoValue, echo_values_equal, php_values_equal};

use super::{EchoArray, EchoArrayKey, echo_value_array_new, echo_value_array_set};

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_values(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        array.values.clone(),
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_keys(
    array: EchoValue,
    filter_value: EchoValue,
    strict: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let strict = strict.bool_value().unwrap_or(false);
    let has_filter = filter_value.kind != ECHO_VALUE_PENDING;

    let mut keys = Vec::new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        if has_filter {
            let matches = if strict {
                echo_values_equal(*value, filter_value)
            } else {
                php_values_equal(*value, filter_value)
            };
            if !matches {
                continue;
            }
        }
        keys.push(key.to_value());
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(keys))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_fill(
    start_index: EchoValue,
    count: EchoValue,
    value: EchoValue,
) -> EchoValue {
    let Some(start_index) = start_index.php_int_value() else {
        return EchoValue::error();
    };
    let Some(count) = count.php_int_value() else {
        return EchoValue::error();
    };
    if !(0..=i32::MAX as i64).contains(&count) {
        return EchoValue::error();
    }

    let mut keys = Vec::with_capacity(count as usize);
    let mut values = Vec::with_capacity(count as usize);
    for offset in 0..count {
        let Some(key) = start_index.checked_add(offset) else {
            return EchoValue::error();
        };
        keys.push(EchoArrayKey::Int(key));
        values.push(value);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_fill_keys(keys: EchoValue, value: EchoValue) -> EchoValue {
    if !keys.is_array() {
        return EchoValue::error();
    }

    let Some(keys_array) = (unsafe { (keys.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut result = echo_value_array_new();
    for key_value in &keys_array.values {
        result = echo_value_array_set(result, *key_value, value);
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_combine(keys: EchoValue, values: EchoValue) -> EchoValue {
    if !keys.is_array() || !values.is_array() {
        return EchoValue::error();
    }

    let Some(keys_array) = (unsafe { (keys.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(values_array) = (unsafe { (values.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    if keys_array.values.len() != values_array.values.len() {
        return EchoValue::error();
    }

    let mut result = echo_value_array_new();
    for (key, value) in keys_array.values.iter().zip(&values_array.values) {
        result = echo_value_array_set(result, *key, *value);
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_key_exists(key: EchoValue, array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(key) = EchoArrayKey::from_value(key) else {
        return EchoValue::error();
    };
    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    EchoValue::bool(array.keys.contains(&key))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_key_first(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    match array.keys.first() {
        Some(key) => key.to_value(),
        None => EchoValue::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_key_last(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    match array.keys.last() {
        Some(key) => key.to_value(),
        None => EchoValue::null(),
    }
}
