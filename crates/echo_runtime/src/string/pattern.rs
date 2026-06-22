use crate::{EchoValue, echo_runtime_string};

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_contains(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    match (haystack.string_bytes(), needle.string_bytes()) {
        (Some(haystack), Some(needle)) => EchoValue::bool(contains_bytes(&haystack, &needle)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_starts_with(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    match (haystack.string_bytes(), needle.string_bytes()) {
        (Some(haystack), Some(needle)) => EchoValue::bool(haystack.starts_with(&needle)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_ends_with(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    match (haystack.string_bytes(), needle.string_bytes()) {
        (Some(haystack), Some(needle)) => EchoValue::bool(haystack.ends_with(&needle)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_replace(
    search: EchoValue,
    replace: EchoValue,
    subject: EchoValue,
) -> EchoValue {
    let Some(search) = search.string_bytes() else {
        return EchoValue::error();
    };
    let Some(replace) = replace.string_bytes() else {
        return EchoValue::error();
    };
    let Some(subject) = subject.string_bytes() else {
        return EchoValue::error();
    };

    echo_runtime_string(replace_bytes(&subject, &search, &replace, false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_ireplace(
    search: EchoValue,
    replace: EchoValue,
    subject: EchoValue,
) -> EchoValue {
    let Some(search) = search.string_bytes() else {
        return EchoValue::error();
    };
    let Some(replace) = replace.string_bytes() else {
        return EchoValue::error();
    };
    let Some(subject) = subject.string_bytes() else {
        return EchoValue::error();
    };

    echo_runtime_string(replace_bytes(&subject, &search, &replace, true))
}

fn replace_bytes(subject: &[u8], search: &[u8], replace: &[u8], case_insensitive: bool) -> Vec<u8> {
    if search.is_empty() {
        return subject.to_vec();
    }

    let mut result = Vec::with_capacity(subject.len());
    let mut index = 0;

    while index < subject.len() {
        let remaining = &subject[index..];
        let matches = remaining.len() >= search.len()
            && if case_insensitive {
                bytes_eq_ascii_case_insensitive(&remaining[..search.len()], search)
            } else {
                &remaining[..search.len()] == search
            };

        if matches {
            result.extend_from_slice(replace);
            index += search.len();
        } else {
            result.push(subject[index]);
            index += 1;
        }
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strtr(value: EchoValue, from: EchoValue, to: EchoValue) -> EchoValue {
    let Some(value) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(from) = from.string_bytes() else {
        return EchoValue::error();
    };
    let Some(to) = to.string_bytes() else {
        return EchoValue::error();
    };

    echo_runtime_string(php_strtr(&value, &from, &to))
}

fn php_strtr(value: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut table = [None; 256];
    for (source, target) in from.iter().copied().zip(to.iter().copied()) {
        table[source as usize] = Some(target);
    }

    value
        .iter()
        .copied()
        .map(|byte| table[byte as usize].unwrap_or(byte))
        .collect()
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strpos(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };

    find_bytes(&haystack, &needle)
        .map(|position| EchoValue::int(position as i64))
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stripos(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };

    find_bytes_ascii_case_insensitive(&haystack, &needle)
        .map(|position| EchoValue::int(position as i64))
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strrpos(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };

    find_last_bytes(&haystack, &needle)
        .map(|position| EchoValue::int(position as i64))
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strripos(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };

    find_last_bytes_ascii_case_insensitive(&haystack, &needle)
        .map(|position| EchoValue::int(position as i64))
        .unwrap_or_else(|| EchoValue::bool(false))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn find_last_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(haystack.len());
    }

    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

fn find_last_bytes_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(haystack.len());
    }

    haystack
        .windows(needle.len())
        .rposition(|window| bytes_eq_ascii_case_insensitive(window, needle))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strstr(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };
    let Some(position) = find_bytes(&haystack, &needle) else {
        return EchoValue::bool(false);
    };

    echo_runtime_string(haystack[position..].to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stristr(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };
    let Some(position) = find_bytes_ascii_case_insensitive(&haystack, &needle) else {
        return EchoValue::bool(false);
    };

    echo_runtime_string(haystack[position..].to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strrchr(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };
    if needle.is_empty() {
        return EchoValue::bool(false);
    }
    let Some(position) = find_last_bytes(&haystack, &needle) else {
        return EchoValue::bool(false);
    };

    echo_runtime_string(haystack[position..].to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strpbrk(value: EchoValue, characters: EchoValue) -> EchoValue {
    let Some(value) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(characters) = characters.string_bytes() else {
        return EchoValue::error();
    };
    if characters.is_empty() {
        return EchoValue::error();
    }
    let Some(position) = value.iter().position(|byte| characters.contains(byte)) else {
        return EchoValue::bool(false);
    };

    echo_runtime_string(value[position..].to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strspn(value: EchoValue, characters: EchoValue) -> EchoValue {
    let Some(value) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(characters) = characters.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::int(
        value
            .iter()
            .take_while(|byte| characters.contains(byte))
            .count() as i64,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strcspn(value: EchoValue, characters: EchoValue) -> EchoValue {
    let Some(value) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(characters) = characters.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::int(
        value
            .iter()
            .take_while(|byte| !characters.contains(byte))
            .count() as i64,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_substr_count(haystack: EchoValue, needle: EchoValue) -> EchoValue {
    let Some(haystack) = haystack.string_bytes() else {
        return EchoValue::error();
    };
    let Some(needle) = needle.string_bytes() else {
        return EchoValue::error();
    };
    if needle.is_empty() {
        return EchoValue::error();
    }

    let mut count = 0;
    let mut offset = 0;
    while offset <= haystack.len().saturating_sub(needle.len()) {
        let Some(position) = find_bytes(&haystack[offset..], &needle) else {
            break;
        };
        count += 1;
        offset += position + needle.len();
    }

    EchoValue::int(count)
}

fn find_bytes_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| bytes_eq_ascii_case_insensitive(window, needle))
}

fn bytes_eq_ascii_case_insensitive(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}
