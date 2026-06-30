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
pub extern "C" fn echo_php_strnatcmp(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::int(natural_compare(&left, &right, false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strnatcasecmp(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::int(natural_compare(&left, &right, true))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_levenshtein(
    string1: EchoValue,
    string2: EchoValue,
    insertion_cost: EchoValue,
    replacement_cost: EchoValue,
    deletion_cost: EchoValue,
) -> EchoValue {
    let Some(string1) = string1.string_bytes() else {
        return EchoValue::error();
    };
    let Some(string2) = string2.string_bytes() else {
        return EchoValue::error();
    };
    let Some(insertion_cost) = non_negative_cost(insertion_cost) else {
        return EchoValue::error();
    };
    let Some(replacement_cost) = non_negative_cost(replacement_cost) else {
        return EchoValue::error();
    };
    let Some(deletion_cost) = non_negative_cost(deletion_cost) else {
        return EchoValue::error();
    };

    EchoValue::int(levenshtein_distance(
        &string1,
        &string2,
        insertion_cost,
        replacement_cost,
        deletion_cost,
    ))
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

fn non_negative_cost(value: EchoValue) -> Option<i64> {
    value.int_value().filter(|cost| *cost >= 0)
}

fn levenshtein_distance(
    source: &[u8],
    target: &[u8],
    insertion_cost: i64,
    replacement_cost: i64,
    deletion_cost: i64,
) -> i64 {
    let mut previous = (0..=target.len())
        .map(|index| (index as i64).saturating_mul(insertion_cost))
        .collect::<Vec<_>>();
    let mut current = vec![0; target.len() + 1];

    for (source_index, source_byte) in source.iter().enumerate() {
        current[0] = ((source_index + 1) as i64).saturating_mul(deletion_cost);

        for (target_index, target_byte) in target.iter().enumerate() {
            let replacement = if source_byte == target_byte {
                previous[target_index]
            } else {
                previous[target_index].saturating_add(replacement_cost)
            };
            let insertion = current[target_index].saturating_add(insertion_cost);
            let deletion = previous[target_index + 1].saturating_add(deletion_cost);

            current[target_index + 1] = replacement.min(insertion).min(deletion);
        }

        std::mem::swap(&mut previous, &mut current);
    }

    previous[target.len()]
}

fn natural_compare(left: &[u8], right: &[u8], case_insensitive: bool) -> i64 {
    let mut left_index = 0;
    let mut right_index = 0;

    while left_index < left.len() && right_index < right.len() {
        let left_byte = left[left_index];
        let right_byte = right[right_index];

        if left_byte.is_ascii_digit() && right_byte.is_ascii_digit() {
            let (ordering, left_end, right_end) =
                compare_natural_number_run(left, left_index, right, right_index);
            if ordering != CmpOrdering::Equal {
                return ordering_to_int(ordering);
            }
            left_index = left_end;
            right_index = right_end;
            continue;
        }

        let left_byte = compare_byte(left_byte, case_insensitive);
        let right_byte = compare_byte(right_byte, case_insensitive);
        if left_byte != right_byte {
            return ordering_to_int(left_byte.cmp(&right_byte));
        }

        left_index += 1;
        right_index += 1;
    }

    ordering_to_int(left.len().cmp(&right.len()))
}

fn compare_natural_number_run(
    left: &[u8],
    left_index: usize,
    right: &[u8],
    right_index: usize,
) -> (CmpOrdering, usize, usize) {
    let left_end = digit_run_end(left, left_index);
    let right_end = digit_run_end(right, right_index);
    let left_significant = first_significant_digit(left, left_index, left_end);
    let right_significant = first_significant_digit(right, right_index, right_end);
    let left_len = left_end - left_significant;
    let right_len = right_end - right_significant;

    let ordering = match left_len.cmp(&right_len) {
        CmpOrdering::Equal => {
            left[left_significant..left_end].cmp(&right[right_significant..right_end])
        }
        other => other,
    };

    if ordering == CmpOrdering::Equal {
        (left_index.cmp(&right_index), left_end, right_end)
    } else {
        (ordering, left_end, right_end)
    }
}

fn digit_run_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    end
}

fn first_significant_digit(bytes: &[u8], start: usize, end: usize) -> usize {
    let significant = bytes[start..end]
        .iter()
        .position(|byte| *byte != b'0')
        .map_or(end - 1, |offset| start + offset);
    significant.min(end)
}

fn compare_byte(byte: u8, case_insensitive: bool) -> u8 {
    if case_insensitive {
        byte.to_ascii_lowercase()
    } else {
        byte
    }
}

fn ordering_to_int(ordering: CmpOrdering) -> i64 {
    match ordering {
        CmpOrdering::Less => -1,
        CmpOrdering::Equal => 0,
        CmpOrdering::Greater => 1,
    }
}
