use crate::{ECHO_VALUE_NULL, EchoValue};

use super::{EchoArray, EchoArrayKey};

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_pad(
    array: EchoValue,
    length: EchoValue,
    value: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let current_len = array.values.len() as i64;
    let target_len = length.checked_abs().unwrap_or(i64::MAX);

    if target_len > i32::MAX as i64 {
        return EchoValue::error();
    }

    if target_len <= current_len {
        return EchoValue::array(Box::into_raw(Box::new(EchoArray {
            keys: array.keys.clone(),
            values: array.values.clone(),
        })));
    }

    let pad_count = (target_len - current_len) as usize;
    let mut keys = Vec::with_capacity(target_len as usize);
    let mut values = Vec::with_capacity(target_len as usize);
    let mut next_index = 0_i64;

    if length < 0 {
        for _ in 0..pad_count {
            keys.push(EchoArrayKey::Int(next_index));
            values.push(value);
            next_index += 1;
        }
    }

    append_array_pad_values(array, &mut keys, &mut values, &mut next_index);

    if length > 0 {
        for _ in 0..pad_count {
            keys.push(EchoArrayKey::Int(next_index));
            values.push(value);
            next_index += 1;
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

fn append_array_pad_values(
    array: &EchoArray,
    keys: &mut Vec<EchoArrayKey>,
    values: &mut Vec<EchoValue>,
    next_index: &mut i64,
) {
    for (key, value) in array.keys.iter().zip(&array.values) {
        match key {
            EchoArrayKey::Int(_) => {
                keys.push(EchoArrayKey::Int(*next_index));
                *next_index += 1;
            }
            EchoArrayKey::String(bytes) => keys.push(EchoArrayKey::String(bytes.clone())),
        }
        values.push(*value);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_reverse(array: EchoValue, preserve_keys: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let preserve_keys = preserve_keys.bool_value().unwrap_or(false);

    let mut keys = Vec::with_capacity(array.keys.len());
    let mut values = Vec::with_capacity(array.values.len());
    let mut next_index = 0_i64;

    for (key, value) in array.keys.iter().zip(&array.values).rev() {
        let key = if preserve_keys {
            key.clone()
        } else {
            match key {
                EchoArrayKey::Int(_) => {
                    let key = EchoArrayKey::Int(next_index);
                    next_index += 1;
                    key
                }
                EchoArrayKey::String(bytes) => EchoArrayKey::String(bytes.clone()),
            }
        };
        keys.push(key);
        values.push(*value);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_slice(
    array: EchoValue,
    offset: EchoValue,
    length: EchoValue,
    preserve_keys: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(offset) = offset.php_int_value() else {
        return EchoValue::error();
    };

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let array_len = array.values.len() as i64;
    let start = if offset < 0 {
        array_len.saturating_add(offset).max(0)
    } else {
        offset.min(array_len)
    };
    let end = match length.kind {
        ECHO_VALUE_NULL => array_len,
        _ => {
            let Some(length) = length.php_int_value() else {
                return EchoValue::error();
            };
            if length < 0 {
                array_len.saturating_add(length).max(start)
            } else {
                start.saturating_add(length).min(array_len)
            }
        }
    };
    let preserve_keys = preserve_keys.bool_value().unwrap_or(false);

    let mut keys = Vec::with_capacity((end - start) as usize);
    let mut values = Vec::with_capacity((end - start) as usize);
    let mut next_index = 0_i64;

    for index in start as usize..end as usize {
        let key = match &array.keys[index] {
            EchoArrayKey::Int(_) if !preserve_keys => {
                let key = EchoArrayKey::Int(next_index);
                next_index += 1;
                key
            }
            key => key.clone(),
        };
        keys.push(key);
        values.push(array.values[index]);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_splice(
    array: EchoValue,
    offset: EchoValue,
    length: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(offset) = offset.php_int_value() else {
        return EchoValue::error();
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };

    let Some(array) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };

    let array_len = array.values.len() as i64;
    let start = if offset < 0 {
        array_len.saturating_add(offset).max(0)
    } else {
        offset.min(array_len)
    };
    let end = if length < 0 {
        array_len.saturating_add(length).max(start)
    } else {
        start.saturating_add(length).min(array_len)
    };

    let removed_values: Vec<EchoValue> = array.values.drain(start as usize..end as usize).collect();
    array.keys.drain(start as usize..end as usize);

    let mut next_index = 0_i64;
    for key in &mut array.keys {
        if matches!(key, EchoArrayKey::Int(_)) {
            *key = EchoArrayKey::Int(next_index);
            next_index += 1;
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        removed_values,
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_chunk(
    array: EchoValue,
    length: EchoValue,
    preserve_keys: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    if length < 1 {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let preserve_keys = preserve_keys.bool_value().unwrap_or(false);

    let chunk_count = array.values.len().div_ceil(length as usize);
    let mut outer_values = Vec::with_capacity(chunk_count);

    for start in (0..array.values.len()).step_by(length as usize) {
        let end = start
            .saturating_add(length as usize)
            .min(array.values.len());
        let mut chunk_keys = Vec::with_capacity(end - start);
        let mut chunk_values = Vec::with_capacity(end - start);

        for (offset, index) in (start..end).enumerate() {
            let key = if preserve_keys {
                array.keys[index].clone()
            } else {
                EchoArrayKey::Int(offset as i64)
            };
            chunk_keys.push(key);
            chunk_values.push(array.values[index]);
        }

        outer_values.push(EchoValue::array(Box::into_raw(Box::new(EchoArray {
            keys: chunk_keys,
            values: chunk_values,
        }))));
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        outer_values,
    ))))
}
