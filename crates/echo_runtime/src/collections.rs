use std::collections::HashSet;

use crate::{EchoString, EchoValue, echo_runtime_string};

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
