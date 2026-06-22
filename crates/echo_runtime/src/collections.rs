use std::collections::HashSet;

mod sequence;

pub use sequence::{
    echo_php_array_chunk, echo_php_array_pad, echo_php_array_reverse, echo_php_array_slice,
};

use crate::{
    ECHO_VALUE_INT, ECHO_VALUE_PENDING, ECHO_VALUE_STRING, EchoString, EchoValue, PhpNumber,
    echo_runtime_string, echo_values_equal, php_number_add, php_number_mul, php_values_equal,
};

#[derive(Debug)]
pub struct EchoList {
    pub(crate) values: Vec<EchoValue>,
}

impl EchoList {
    pub(crate) fn new() -> Self {
        Self { values: Vec::new() }
    }
}

#[derive(Debug)]
pub struct EchoArray {
    pub(crate) keys: Vec<EchoArrayKey>,
    pub(crate) values: Vec<EchoValue>,
}

impl EchoArray {
    pub(crate) fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub(crate) fn from_values(values: Vec<EchoValue>) -> Self {
        let keys = (0..values.len())
            .map(|key| EchoArrayKey::Int(key as i64))
            .collect();
        Self { keys, values }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum EchoArrayKey {
    Int(i64),
    String(Vec<u8>),
}

impl EchoArrayKey {
    pub(crate) fn from_value(value: EchoValue) -> Option<Self> {
        if value.is_int() {
            return Some(Self::Int(value.payload as i64));
        }

        if value.is_float() {
            return Some(Self::Int(f64::from_bits(value.payload) as i64));
        }

        if value.is_bool() {
            return Some(Self::Int(if value.payload == 0 { 0 } else { 1 }));
        }

        if value.is_null() {
            return Some(Self::String(Vec::new()));
        }

        if value.is_string() {
            let bytes = unsafe { &(value.payload as *const EchoString).as_ref()?.bytes };
            return match parse_php_array_integer_key(bytes) {
                Some(key) => Some(Self::Int(key)),
                None => Some(Self::String(bytes.clone())),
            };
        }

        None
    }

    pub(crate) fn to_value(&self) -> EchoValue {
        match self {
            Self::Int(value) => EchoValue::int(*value),
            Self::String(bytes) => echo_runtime_string(bytes.clone()),
        }
    }
}

pub(crate) fn php_array_union(left: EchoValue, right: EchoValue) -> EchoValue {
    if !left.is_array() || !right.is_array() {
        return EchoValue::error();
    }

    let Some(left) = (unsafe { (left.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(right) = (unsafe { (right.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut seen_keys: HashSet<EchoArrayKey> = left.keys.iter().cloned().collect();
    let mut keys = left.keys.clone();
    let mut values = left.values.clone();
    for (key, value) in right.keys.iter().zip(&right.values) {
        if seen_keys.insert(key.clone()) {
            keys.push(key.clone());
            values.push(*value);
        }
    }
    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

pub(crate) fn echo_arrays_equal(
    left: EchoValue,
    right: EchoValue,
    values_equal: fn(EchoValue, EchoValue) -> bool,
) -> bool {
    let Some(left) = (unsafe { (left.payload as *const EchoArray).as_ref() }) else {
        return false;
    };
    let Some(right) = (unsafe { (right.payload as *const EchoArray).as_ref() }) else {
        return false;
    };

    left.values.len() == right.values.len()
        && left
            .values
            .iter()
            .zip(&right.values)
            .all(|(left, right)| values_equal(*left, *right))
}

pub(crate) fn echo_lists_equal(
    left: EchoValue,
    right: EchoValue,
    values_equal: fn(EchoValue, EchoValue) -> bool,
) -> bool {
    let Some(left) = (unsafe { (left.payload as *const EchoList).as_ref() }) else {
        return false;
    };
    let Some(right) = (unsafe { (right.payload as *const EchoList).as_ref() }) else {
        return false;
    };

    left.values.len() == right.values.len()
        && left
            .values
            .iter()
            .zip(&right.values)
            .all(|(left, right)| values_equal(*left, *right))
}

pub(crate) fn next_array_append_key(array: &EchoArray) -> EchoArrayKey {
    let next = array
        .keys
        .iter()
        .filter_map(|key| match key {
            EchoArrayKey::Int(value) => Some(*value),
            EchoArrayKey::String(_) => None,
        })
        .filter(|value| *value >= 0)
        .max()
        .map(|value| value.saturating_add(1))
        .unwrap_or(0);
    EchoArrayKey::Int(next)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_list_new() -> EchoValue {
    EchoValue::list(Box::into_raw(Box::new(EchoList::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_list_append(list: EchoValue, value: EchoValue) -> EchoValue {
    if !list.is_list() {
        return EchoValue::error();
    }

    let Some(values) = (unsafe { (list.payload as *mut EchoList).as_mut() }) else {
        return EchoValue::error();
    };

    values.values.push(value);
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_array_new() -> EchoValue {
    EchoValue::array(Box::into_raw(Box::new(EchoArray::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_array_append(array: EchoValue, value: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(values) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };

    values.keys.push(next_array_append_key(values));
    values.values.push(value);
    array
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_array_set(
    array: EchoValue,
    key: EchoValue,
    value: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(key) = EchoArrayKey::from_value(key) else {
        return EchoValue::error();
    };

    let Some(values) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };

    if let Some(index) = values.keys.iter().position(|existing| existing == &key) {
        values.values[index] = value;
    } else {
        values.keys.push(key);
        values.values.push(value);
    }
    array
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_index_get(collection: EchoValue, index: EchoValue) -> EchoValue {
    if collection.is_array() {
        let Some(key) = EchoArrayKey::from_value(index) else {
            return EchoValue::null();
        };
        let Some(array) = (unsafe { (collection.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::null();
        };

        return array
            .keys
            .iter()
            .position(|existing| existing == &key)
            .map(|position| array.values[position])
            .unwrap_or_else(EchoValue::null);
    }

    if collection.is_list() {
        let Some(index) = index.int_value() else {
            return EchoValue::null();
        };
        if index < 0 {
            return EchoValue::null();
        }
        let Some(list) = (unsafe { (collection.payload as *const EchoList).as_ref() }) else {
            return EchoValue::null();
        };

        return list
            .values
            .get(index as usize)
            .copied()
            .unwrap_or_else(EchoValue::null);
    }

    EchoValue::error()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_count(value: EchoValue) -> EchoValue {
    if value.is_array() {
        let Some(array) = (unsafe { (value.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::error();
        };
        return EchoValue::int(array.values.len() as i64);
    }

    if value.is_list() {
        let Some(list) = (unsafe { (value.payload as *const EchoList).as_ref() }) else {
            return EchoValue::error();
        };
        return EchoValue::int(list.values.len() as i64);
    }

    EchoValue::error()
}

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
pub extern "C" fn echo_php_array_merge(arrays: EchoValue) -> EchoValue {
    if !arrays.is_array() {
        return EchoValue::error();
    }

    let Some(arrays) = (unsafe { (arrays.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut next_index = 0_i64;

    for array_value in &arrays.values {
        if !array_value.is_array() {
            return EchoValue::error();
        }
        let Some(array) = (unsafe { (array_value.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::error();
        };

        for (key, value) in array.keys.iter().zip(&array.values) {
            match key {
                EchoArrayKey::Int(_) => {
                    keys.push(EchoArrayKey::Int(next_index));
                    values.push(*value);
                    next_index += 1;
                }
                EchoArrayKey::String(bytes) => {
                    let key = EchoArrayKey::String(bytes.clone());
                    match keys.iter().position(|existing| existing == &key) {
                        Some(index) => values[index] = *value,
                        None => {
                            keys.push(key);
                            values.push(*value);
                        }
                    }
                }
            }
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_replace(arrays: EchoValue) -> EchoValue {
    if !arrays.is_array() {
        return EchoValue::error();
    }

    let Some(arrays) = (unsafe { (arrays.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    if arrays.values.is_empty() {
        return EchoValue::error();
    }

    let mut result = echo_value_array_new();
    for array_value in &arrays.values {
        if !array_value.is_array() {
            return EchoValue::error();
        }
        let Some(array) = (unsafe { (array_value.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::error();
        };

        for (key, value) in array.keys.iter().zip(&array.values) {
            result = echo_value_array_set(result, key.to_value(), *value);
        }
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_flip(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut result = echo_value_array_new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        if !matches!(value.kind, ECHO_VALUE_INT | ECHO_VALUE_STRING) {
            continue;
        }
        result = echo_value_array_set(result, *value, key.to_value());
    }

    result
}

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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_sum(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut total = PhpNumber::Int(0);
    for value in &array.values {
        total = php_number_add(
            total,
            PhpNumber::coerce(*value).unwrap_or(PhpNumber::Int(0)),
        );
    }

    total.into_echo_value()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_product(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut product = PhpNumber::Int(1);
    for value in &array.values {
        product = php_number_mul(
            product,
            PhpNumber::coerce(*value).unwrap_or(PhpNumber::Int(0)),
        );
    }

    product.into_echo_value()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_is_list(value: EchoValue) -> EchoValue {
    if !value.is_array() {
        return EchoValue::bool(false);
    }
    let Some(array) = (unsafe { (value.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    EchoValue::bool(
        array
            .keys
            .iter()
            .enumerate()
            .all(|(index, key)| key == &EchoArrayKey::Int(index as i64)),
    )
}

fn parse_php_array_integer_key(bytes: &[u8]) -> Option<i64> {
    let text = std::str::from_utf8(bytes).ok()?;
    if text == "0" {
        return Some(0);
    }
    if let Some(rest) = text.strip_prefix('-') {
        if rest.starts_with('0') || rest.is_empty() {
            return None;
        }
    } else if text.starts_with('0') || text.is_empty() {
        return None;
    }
    text.parse::<i64>().ok()
}
