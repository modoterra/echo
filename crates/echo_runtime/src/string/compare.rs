use crate::EchoValue;
use std::cmp::Ordering as CmpOrdering;

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_substr_compare(
    haystack: EchoValue,
    needle: EchoValue,
    offset: EchoValue,
    length: EchoValue,
    case_insensitive: EchoValue,
) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };
    let Some(offset) = offset.int_value() else {
        return EchoValue::error();
    };
    let Some(case_insensitive) = case_insensitive.bool_value() else {
        return EchoValue::error();
    };

    let start = if offset < 0 {
        let start = haystack.len() as i64 + offset;
        if start < 0 {
            return EchoValue::bool(false);
        }
        start as usize
    } else {
        offset as usize
    };

    if start > haystack.len() {
        return EchoValue::bool(false);
    }

    let default_length = needle.len().max(haystack.len().saturating_sub(start));
    let length = if length.is_null() {
        default_length
    } else {
        let Some(length) = length.int_value() else {
            return EchoValue::error();
        };
        let Ok(length) = usize::try_from(length) else {
            return EchoValue::bool(false);
        };
        length
    };

    let haystack = &haystack[start..haystack.len().min(start + length)];
    let needle = &needle[..needle.len().min(length)];
    if case_insensitive {
        EchoValue::int(case_insensitive_ascii_compare(haystack, needle))
    } else {
        EchoValue::int(match haystack.cmp(needle) {
            CmpOrdering::Less => -1,
            CmpOrdering::Equal => 0,
            CmpOrdering::Greater => 1,
        })
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strcmp(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::int(match left.cmp(&right) {
        CmpOrdering::Less => -1,
        CmpOrdering::Equal => 0,
        CmpOrdering::Greater => 1,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strcasecmp(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::int(case_insensitive_ascii_compare(&left, &right))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strncmp(
    left: EchoValue,
    right: EchoValue,
    length: EchoValue,
) -> EchoValue {
    let Some(left) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.int_value() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::error();
    };

    EchoValue::int(
        match left[..left.len().min(length)].cmp(&right[..right.len().min(length)]) {
            CmpOrdering::Less => -1,
            CmpOrdering::Equal => 0,
            CmpOrdering::Greater => 1,
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strncasecmp(
    left: EchoValue,
    right: EchoValue,
    length: EchoValue,
) -> EchoValue {
    let Some(left) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.int_value() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::error();
    };

    EchoValue::int(case_insensitive_ascii_compare(
        &left[..left.len().min(length)],
        &right[..right.len().min(length)],
    ))
}

fn case_insensitive_ascii_compare(left: &[u8], right: &[u8]) -> i64 {
    for (left, right) in left.iter().zip(right) {
        let left = left.to_ascii_lowercase();
        let right = right.to_ascii_lowercase();

        if left != right {
            return left as i64 - right as i64;
        }
    }

    match left.len().cmp(&right.len()) {
        CmpOrdering::Less => -1,
        CmpOrdering::Equal => 0,
        CmpOrdering::Greater => 1,
    }
}
