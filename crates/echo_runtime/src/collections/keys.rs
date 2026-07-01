use crate::{ECHO_VALUE_PENDING, EchoValue, echo_values_equal, php_values_equal};

use super::{
    EchoArray, EchoArrayKey, echo_value_array_new, echo_value_array_set, next_array_append_key,
};

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
pub extern "C" fn echo_php_array_change_key_case(array: EchoValue, case: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(case) = case.php_int_value() else {
        return EchoValue::error();
    };
    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let keys = array
        .keys
        .iter()
        .map(|key| match key {
            EchoArrayKey::Int(value) => EchoArrayKey::Int(*value),
            EchoArrayKey::String(bytes) if case == 1 => {
                EchoArrayKey::String(bytes.iter().map(u8::to_ascii_uppercase).collect())
            }
            EchoArrayKey::String(bytes) => {
                EchoArrayKey::String(bytes.iter().map(u8::to_ascii_lowercase).collect())
            }
        })
        .collect();

    EchoValue::array(Box::into_raw(Box::new(EchoArray {
        keys,
        values: array.values.clone(),
    })))
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
pub extern "C" fn echo_php_array_diff_key(array: EchoValue, other: EchoValue) -> EchoValue {
    if !array.is_array() || !other.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(other) = (unsafe { (other.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        if !other.keys.contains(key) {
            keys.push(key.clone());
            values.push(*value);
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_intersect_key(array: EchoValue, other: EchoValue) -> EchoValue {
    if !array.is_array() || !other.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(other) = (unsafe { (other.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        if other.keys.contains(key) {
            keys.push(key.clone());
            values.push(*value);
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_first(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    array
        .values
        .first()
        .copied()
        .unwrap_or_else(EchoValue::null)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_last(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    array.values.last().copied().unwrap_or_else(EchoValue::null)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_pop(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };

    array.keys.pop();
    array.values.pop().unwrap_or_else(EchoValue::null)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_push(array: EchoValue, value: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };

    array.keys.push(next_array_append_key(array));
    array.values.push(value);
    EchoValue::int(array.values.len() as i64)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_shift(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };
    if array.values.is_empty() {
        return EchoValue::null();
    }

    array.keys.remove(0);
    let value = array.values.remove(0);
    let mut next_index = 0_i64;
    for key in &mut array.keys {
        if matches!(key, EchoArrayKey::Int(_)) {
            *key = EchoArrayKey::Int(next_index);
            next_index += 1;
        }
    }

    value
}
