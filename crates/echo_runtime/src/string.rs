use crate::encoding::{quoted_printable_decode_bytes, quoted_printable_encode_bytes};
use crate::{
    EchoString, EchoValue, echo_runtime_string, echo_value_array_new, echo_value_array_set,
};

mod compare;
mod pattern;
mod sequence;

pub use compare::{
    echo_php_levenshtein, echo_php_strcasecmp, echo_php_strcmp, echo_php_strnatcasecmp,
    echo_php_strnatcmp, echo_php_strncasecmp, echo_php_strncmp, echo_php_substr_compare,
};

pub use pattern::{
    echo_php_str_contains, echo_php_str_ends_with, echo_php_str_ireplace, echo_php_str_replace,
    echo_php_str_starts_with, echo_php_strcspn, echo_php_stripos, echo_php_stristr,
    echo_php_strpbrk, echo_php_strpos, echo_php_strrchr, echo_php_strripos, echo_php_strrpos,
    echo_php_strspn, echo_php_strstr, echo_php_strtr, echo_php_substr_count,
};
pub use sequence::{echo_php_chunk_split, echo_php_explode, echo_php_implode, echo_php_str_split};

const PHP_DEFAULT_TRIM_BYTES: &[u8] = b" \n\r\t\x0b\0";

pub(crate) fn php_string_transform_builtin(
    value: EchoValue,
    f: impl FnOnce(&mut Vec<u8>),
) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            f(&mut bytes);
            echo_runtime_string(bytes)
        }
        None => EchoValue::error(),
    }
}

pub(crate) fn php_string_map_builtin(
    value: EchoValue,
    f: impl FnOnce(&[u8]) -> Vec<u8>,
) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(f(&bytes)),
        None => EchoValue::error(),
    }
}

pub(crate) fn php_int_to_string_builtin(
    value: EchoValue,
    f: impl FnOnce(i64) -> Vec<u8>,
) -> EchoValue {
    match value.php_int_value() {
        Some(number) => echo_runtime_string(f(number)),
        None => EchoValue::error(),
    }
}

pub(crate) fn php_string_to_number_builtin(
    value: EchoValue,
    f: impl FnOnce(&[u8]) -> EchoValue,
) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => f(&bytes),
        None => EchoValue::error(),
    }
}

pub(crate) fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let bytes = trim_ascii_start(bytes);
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(0, |index| index + 1);
    &bytes[..end]
}

pub(crate) fn trim_ascii_start(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    &bytes[start..]
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_string(ptr: *const u8, len: usize) -> EchoValue {
    if ptr.is_null() && len != 0 {
        return EchoValue::error();
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_concat(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(mut bytes) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };

    bytes.extend_from_slice(&right);
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strlen(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::int(bytes.len() as i64),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_word_count(value: EchoValue) -> EchoValue {
    php_string_to_number_builtin(value, |bytes| EchoValue::int(count_php_words(bytes) as i64))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_count_chars(value: EchoValue, mode: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(mode) = mode.php_int_value() else {
        return EchoValue::error();
    };

    let mut counts = [0_i64; 256];
    for byte in bytes {
        counts[byte as usize] += 1;
    }

    match mode {
        0 => count_chars_array(&counts, |_| true),
        1 => count_chars_array(&counts, |count| count > 0),
        2 => count_chars_array(&counts, |count| count == 0),
        3 => count_chars_string(&counts, |count| count > 0),
        4 => count_chars_string(&counts, |count| count == 0),
        _ => EchoValue::error(),
    }
}

fn count_chars_array(counts: &[i64; 256], include: impl Fn(i64) -> bool) -> EchoValue {
    let mut result = echo_value_array_new();
    for (byte, count) in counts.iter().enumerate() {
        if include(*count) {
            result =
                echo_value_array_set(result, EchoValue::int(byte as i64), EchoValue::int(*count));
        }
    }
    result
}

fn count_chars_string(counts: &[i64; 256], include: impl Fn(i64) -> bool) -> EchoValue {
    let bytes = counts
        .iter()
        .enumerate()
        .filter_map(|(byte, count)| include(*count).then_some(byte as u8))
        .collect();
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_wordwrap(
    string: EchoValue,
    width: EchoValue,
    break_string: EchoValue,
    cut_long_words: EchoValue,
) -> EchoValue {
    let Some(string) = string.string_bytes() else {
        return EchoValue::error();
    };
    let Some(width) = width.int_value() else {
        return EchoValue::error();
    };
    let Ok(width) = usize::try_from(width) else {
        return EchoValue::error();
    };
    let Some(break_string) = break_string.string_bytes() else {
        return EchoValue::error();
    };
    if break_string.is_empty() {
        return EchoValue::error();
    }
    let Some(cut_long_words) = cut_long_words.bool_value() else {
        return EchoValue::error();
    };

    echo_runtime_string(wordwrap_bytes(
        &string,
        width,
        &break_string,
        cut_long_words,
    ))
}

fn count_php_words(bytes: &[u8]) -> usize {
    let mut count = 0;
    let mut in_word = false;

    for (index, byte) in bytes.iter().copied().enumerate() {
        let word_byte = byte.is_ascii_alphabetic()
            || ((byte == b'\'' || byte == b'-')
                && in_word
                && bytes
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_alphabetic()));

        if word_byte {
            if !in_word {
                count += 1;
                in_word = true;
            }
        } else {
            in_word = false;
        }
    }

    count
}

fn wordwrap_bytes(
    bytes: &[u8],
    width: usize,
    break_string: &[u8],
    cut_long_words: bool,
) -> Vec<u8> {
    if width == 0 || bytes.is_empty() {
        return bytes.to_vec();
    }

    let mut output = Vec::with_capacity(bytes.len());
    let mut first_line = true;

    for line in bytes.split(|byte| *byte == b'\n') {
        if !first_line {
            output.push(b'\n');
        }
        first_line = false;
        wrap_line(line, width, break_string, cut_long_words, &mut output);
    }

    output
}

fn wrap_line(
    mut line: &[u8],
    width: usize,
    break_string: &[u8],
    cut_long_words: bool,
    output: &mut Vec<u8>,
) {
    let mut first_segment = true;

    while line.len() > width {
        if let Some(space_index) = line[..=width.min(line.len() - 1)]
            .iter()
            .rposition(|byte| *byte == b' ')
        {
            push_wrapped_segment(
                &line[..space_index],
                break_string,
                &mut first_segment,
                output,
            );
            line = trim_leading_spaces(&line[space_index + 1..]);
            continue;
        }

        if cut_long_words {
            push_wrapped_segment(&line[..width], break_string, &mut first_segment, output);
            line = &line[width..];
            continue;
        }

        if let Some(space_index) = line[width..].iter().position(|byte| *byte == b' ') {
            let break_index = width + space_index;
            push_wrapped_segment(
                &line[..break_index],
                break_string,
                &mut first_segment,
                output,
            );
            line = trim_leading_spaces(&line[break_index + 1..]);
        } else {
            break;
        }
    }

    push_wrapped_segment(line, break_string, &mut first_segment, output);
}

fn push_wrapped_segment(
    segment: &[u8],
    break_string: &[u8],
    first_segment: &mut bool,
    output: &mut Vec<u8>,
) {
    if !*first_segment {
        output.extend_from_slice(break_string);
    }
    *first_segment = false;
    output.extend_from_slice(segment);
}

fn trim_leading_spaces(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| *byte != b' ')
        .unwrap_or(bytes.len());
    &bytes[start..]
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_trim(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| trim_bytes(bytes, true, true))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ltrim(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| trim_bytes(bytes, true, false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rtrim(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| trim_bytes(bytes, false, true))
}

fn trim_bytes(bytes: &[u8], left: bool, right: bool) -> Vec<u8> {
    let mut start = 0;
    let mut end = bytes.len();

    if left {
        while start < end && PHP_DEFAULT_TRIM_BYTES.contains(&bytes[start]) {
            start += 1;
        }
    }

    if right {
        while end > start && PHP_DEFAULT_TRIM_BYTES.contains(&bytes[end - 1]) {
            end -= 1;
        }
    }

    bytes[start..end].to_vec()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_chr(value: EchoValue) -> EchoValue {
    match value.int_value() {
        Some(codepoint) => {
            let byte = codepoint.rem_euclid(256) as u8;
            echo_runtime_string(vec![byte])
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_dechex(value: EchoValue) -> EchoValue {
    php_int_to_string_builtin(value, |number| format!("{:x}", number as u64).into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_decbin(value: EchoValue) -> EchoValue {
    php_int_to_string_builtin(value, |number| format!("{:b}", number as u64).into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_decoct(value: EchoValue) -> EchoValue {
    php_int_to_string_builtin(value, |number| format!("{:o}", number as u64).into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strval(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(bytes),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strtoupper(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| bytes.make_ascii_uppercase())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strtolower(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| bytes.make_ascii_lowercase())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ucwords(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        let mut uppercase_next = true;
        for byte in bytes {
            if uppercase_next {
                byte.make_ascii_uppercase();
            }
            uppercase_next = matches!(*byte, b' ' | b'\t' | b'\r' | b'\n' | 0x0c | 0x0b);
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strrev(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| bytes.reverse())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ucfirst(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        if let Some(first) = bytes.first_mut() {
            first.make_ascii_uppercase();
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_lcfirst(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        if let Some(first) = bytes.first_mut() {
            first.make_ascii_lowercase();
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ord(value: EchoValue) -> EchoValue {
    match value
        .string_bytes()
        .and_then(|bytes| bytes.first().copied())
    {
        Some(byte) => EchoValue::int(byte as i64),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_rot13(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        for byte in bytes {
            *byte = match *byte {
                b'a'..=b'm' | b'A'..=b'M' => *byte + 13,
                b'n'..=b'z' | b'N'..=b'Z' => *byte - 13,
                other => other,
            };
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hebrev(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| bytes.to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_soundex(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, soundex_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_similar_text(first: EchoValue, second: EchoValue) -> EchoValue {
    let Some(first) = first.string_bytes() else {
        return EchoValue::error();
    };
    let Some(second) = second.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::int(similar_text_count(&first, &second) as i64)
}

fn soundex_bytes(bytes: &[u8]) -> Vec<u8> {
    let Some((first_index, first_letter)) = bytes
        .iter()
        .copied()
        .enumerate()
        .find(|(_, byte)| byte.is_ascii_alphabetic())
    else {
        return b"0000".to_vec();
    };

    let mut code = Vec::with_capacity(4);
    code.push(first_letter.to_ascii_uppercase());

    let mut previous_digit = soundex_digit(first_letter);
    for byte in bytes[first_index + 1..].iter().copied() {
        if !byte.is_ascii_alphabetic() {
            previous_digit = None;
            continue;
        }
        if matches!(byte.to_ascii_uppercase(), b'H' | b'W') {
            continue;
        }

        let digit = soundex_digit(byte);
        if let Some(digit) = digit
            && Some(digit) != previous_digit
            && code.len() < 4
        {
            code.push(digit);
        }

        previous_digit = digit;
        if code.len() == 4 {
            break;
        }
    }

    while code.len() < 4 {
        code.push(b'0');
    }

    code
}

fn soundex_digit(byte: u8) -> Option<u8> {
    match byte.to_ascii_uppercase() {
        b'B' | b'F' | b'P' | b'V' => Some(b'1'),
        b'C' | b'G' | b'J' | b'K' | b'Q' | b'S' | b'X' | b'Z' => Some(b'2'),
        b'D' | b'T' => Some(b'3'),
        b'L' => Some(b'4'),
        b'M' | b'N' => Some(b'5'),
        b'R' => Some(b'6'),
        _ => None,
    }
}

fn similar_text_count(first: &[u8], second: &[u8]) -> usize {
    let mut best_first = 0;
    let mut best_second = 0;
    let mut best_len = 0;

    for first_index in 0..first.len() {
        for second_index in 0..second.len() {
            let mut len = 0;
            while first_index + len < first.len()
                && second_index + len < second.len()
                && first[first_index + len] == second[second_index + len]
            {
                len += 1;
            }
            if len > best_len {
                best_first = first_index;
                best_second = second_index;
                best_len = len;
            }
        }
    }

    if best_len == 0 {
        return 0;
    }

    best_len
        + similar_text_count(&first[..best_first], &second[..best_second])
        + similar_text_count(
            &first[best_first + best_len..],
            &second[best_second + best_len..],
        )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_repeat(value: EchoValue, times: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(times) = times.int_value() else {
        return EchoValue::error();
    };
    let Ok(times) = usize::try_from(times) else {
        return EchoValue::error();
    };

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        bytes.repeat(times),
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_addslashes(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        let mut escaped = Vec::with_capacity(bytes.len());
        for byte in bytes {
            match byte {
                b'\'' | b'"' | b'\\' => {
                    escaped.push(b'\\');
                    escaped.push(*byte);
                }
                b'\0' => escaped.extend_from_slice(b"\\0"),
                other => escaped.push(*other),
            }
        }
        escaped
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_addcslashes(value: EchoValue, characters: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(characters) = characters.string_bytes() else {
        return EchoValue::error();
    };

    echo_runtime_string(add_c_slashes(&bytes, &characters))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stripslashes(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        let mut stripped = Vec::with_capacity(bytes.len());
        let mut index = 0;

        while index < bytes.len() {
            if bytes[index] != b'\\' || index + 1 == bytes.len() {
                stripped.push(bytes[index]);
                index += 1;
                continue;
            }

            match bytes[index + 1] {
                b'0' => stripped.push(b'\0'),
                other => stripped.push(other),
            }
            index += 2;
        }

        stripped
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stripcslashes(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, strip_c_string_slashes)
}

fn add_c_slashes(bytes: &[u8], characters: &[u8]) -> Vec<u8> {
    let mask = c_slash_charlist_mask(characters);
    let mut escaped = Vec::with_capacity(bytes.len());

    for byte in bytes {
        if !mask[*byte as usize] {
            escaped.push(*byte);
            continue;
        }

        match *byte {
            b'\n' => escaped.extend_from_slice(b"\\n"),
            b'\r' => escaped.extend_from_slice(b"\\r"),
            b'\t' => escaped.extend_from_slice(b"\\t"),
            0x07 => escaped.extend_from_slice(b"\\a"),
            0x08 => escaped.extend_from_slice(b"\\b"),
            0x0b => escaped.extend_from_slice(b"\\v"),
            0x0c => escaped.extend_from_slice(b"\\f"),
            b'\\' => escaped.extend_from_slice(b"\\\\"),
            0x20..=0x7e => {
                escaped.push(b'\\');
                escaped.push(*byte);
            }
            other => escaped.extend_from_slice(format!("\\{:03o}", other).as_bytes()),
        }
    }

    escaped
}

fn c_slash_charlist_mask(characters: &[u8]) -> [bool; 256] {
    let mut mask = [false; 256];
    let mut index = 0;

    while index < characters.len() {
        let (start, consumed) = c_slash_charlist_byte(&characters[index..]);
        index += consumed;

        if index + 1 < characters.len()
            && characters[index] == b'.'
            && characters[index + 1] == b'.'
        {
            index += 2;
            let (end, end_consumed) = c_slash_charlist_byte(&characters[index..]);
            index += end_consumed;

            if start <= end {
                for byte in start..=end {
                    mask[byte as usize] = true;
                }
            } else {
                mask[start as usize] = true;
                mask[end as usize] = true;
            }
        } else {
            mask[start as usize] = true;
        }
    }

    mask
}

fn c_slash_charlist_byte(bytes: &[u8]) -> (u8, usize) {
    if bytes.first() != Some(&b'\\') || bytes.len() == 1 {
        return (bytes[0], 1);
    }

    match bytes[1] {
        b'n' => (b'\n', 2),
        b'r' => (b'\r', 2),
        b't' => (b'\t', 2),
        b'v' => (0x0b, 2),
        b'f' => (0x0c, 2),
        b'a' => (0x07, 2),
        b'b' => (0x08, 2),
        b'0'..=b'7' => {
            let mut value = 0_u8;
            let mut consumed = 1;
            while consumed < bytes.len() && consumed <= 3 && matches!(bytes[consumed], b'0'..=b'7')
            {
                value = value
                    .saturating_mul(8)
                    .saturating_add(bytes[consumed] - b'0');
                consumed += 1;
            }
            (value, consumed)
        }
        other => (other, 2),
    }
}

fn strip_c_string_slashes(bytes: &[u8]) -> Vec<u8> {
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'\\' || index + 1 == bytes.len() {
            stripped.push(bytes[index]);
            index += 1;
            continue;
        }

        let escaped = bytes[index + 1];
        match escaped {
            b'n' => {
                stripped.push(b'\n');
                index += 2;
            }
            b'r' => {
                stripped.push(b'\r');
                index += 2;
            }
            b't' => {
                stripped.push(b'\t');
                index += 2;
            }
            b'v' => {
                stripped.push(0x0b);
                index += 2;
            }
            b'f' => {
                stripped.push(0x0c);
                index += 2;
            }
            b'a' => {
                stripped.push(0x07);
                index += 2;
            }
            b'b' => {
                stripped.push(0x08);
                index += 2;
            }
            b'0'..=b'7' => {
                let (byte, consumed) = parse_c_octal_escape(&bytes[index + 1..]);
                stripped.push(byte);
                index += 1 + consumed;
            }
            b'x' => {
                let (byte, consumed) = parse_c_hex_escape(&bytes[index + 2..]);
                if consumed == 0 {
                    stripped.push(b'x');
                    index += 2;
                } else {
                    stripped.push(byte);
                    index += 2 + consumed;
                }
            }
            other => {
                stripped.push(other);
                index += 2;
            }
        }
    }

    stripped
}

fn parse_c_octal_escape(bytes: &[u8]) -> (u8, usize) {
    let mut value = 0u16;
    let mut consumed = 0;

    for byte in bytes.iter().copied().take(3) {
        if !(b'0'..=b'7').contains(&byte) {
            break;
        }
        value = value * 8 + u16::from(byte - b'0');
        consumed += 1;
    }

    (value as u8, consumed)
}

fn parse_c_hex_escape(bytes: &[u8]) -> (u8, usize) {
    let mut value = 0u16;
    let mut consumed = 0;

    for byte in bytes.iter().copied().take(2) {
        let Some(digit) = ascii_hex_digit(byte) else {
            break;
        };
        value = value * 16 + u16::from(digit);
        consumed += 1;
    }

    (value as u8, consumed)
}

fn ascii_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quoted_printable_encode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, quoted_printable_encode_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quoted_printable_decode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, quoted_printable_decode_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_htmlspecialchars(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        let mut escaped = Vec::with_capacity(bytes.len());
        for byte in bytes {
            match byte {
                b'&' => escaped.extend_from_slice(b"&amp;"),
                b'"' => escaped.extend_from_slice(b"&quot;"),
                b'\'' => escaped.extend_from_slice(b"&#039;"),
                b'<' => escaped.extend_from_slice(b"&lt;"),
                b'>' => escaped.extend_from_slice(b"&gt;"),
                other => escaped.push(*other),
            }
        }
        escaped
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_htmlentities(value: EchoValue) -> EchoValue {
    echo_php_htmlspecialchars(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_htmlspecialchars_decode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, php_htmlspecialchars_decode)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_html_entity_decode(value: EchoValue) -> EchoValue {
    echo_php_htmlspecialchars_decode(value)
}

fn php_htmlspecialchars_decode(bytes: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        let remaining = &bytes[index..];
        let decoded_byte = if remaining.starts_with(b"&amp;") {
            Some((b'&', 5))
        } else if remaining.starts_with(b"&quot;") {
            Some((b'"', 6))
        } else if remaining.starts_with(b"&#039;") {
            Some((b'\'', 6))
        } else if remaining.starts_with(b"&lt;") {
            Some((b'<', 4))
        } else if remaining.starts_with(b"&gt;") {
            Some((b'>', 4))
        } else {
            None
        };

        if let Some((byte, consumed)) = decoded_byte {
            decoded.push(byte);
            index += consumed;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    decoded
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strip_tags(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, php_strip_tags_default)
}

fn php_strip_tags_default(bytes: &[u8]) -> Vec<u8> {
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\0' => index += 1,
            b'<' => {
                if let Some(close_offset) = bytes[index + 1..].iter().position(|byte| *byte == b'>')
                {
                    index += close_offset + 2;
                } else {
                    stripped.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                stripped.push(byte);
                index += 1;
            }
        }
    }

    stripped
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_nl2br(value: EchoValue, use_xhtml: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let use_xhtml = use_xhtml.bool_value().unwrap_or(true);
    let marker: &[u8] = if use_xhtml { b"<br />" } else { b"<br>" };

    echo_runtime_string(php_nl2br(&bytes, marker))
}

fn php_nl2br(bytes: &[u8], marker: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                result.extend_from_slice(marker);
                result.extend_from_slice(b"\r\n");
                index += 2;
            }
            b'\n' => {
                result.extend_from_slice(marker);
                result.push(b'\n');
                index += 1;
            }
            b'\r' => {
                result.extend_from_slice(marker);
                result.push(b'\r');
                index += 1;
            }
            other => {
                result.push(other);
                index += 1;
            }
        }
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quotemeta(value: EchoValue) -> EchoValue {
    const META_BYTES: &[u8] = b".\\+*?[^]($)";

    match value.string_bytes() {
        Some(bytes) if bytes.is_empty() => EchoValue::bool(false),
        Some(bytes) => {
            let mut quoted = Vec::with_capacity(bytes.len());
            for byte in bytes {
                if META_BYTES.contains(&byte) {
                    quoted.push(b'\\');
                }
                quoted.push(byte);
            }
            echo_runtime_string(quoted)
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_pad(
    value: EchoValue,
    length: EchoValue,
    pad_string: EchoValue,
    pad_type: EchoValue,
) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    let Some(pad_string) = pad_string.string_bytes() else {
        return EchoValue::error();
    };
    let Some(pad_type) = pad_type.php_int_value() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return echo_runtime_string(bytes);
    };
    if pad_string.is_empty() {
        return EchoValue::error();
    }

    echo_runtime_string(php_str_pad(&bytes, length, &pad_string, pad_type))
}

fn php_str_pad(bytes: &[u8], length: usize, pad_string: &[u8], pad_type: i64) -> Vec<u8> {
    let missing = length.saturating_sub(bytes.len());
    if missing == 0 {
        return bytes.to_vec();
    }

    let (left, right) = match pad_type {
        0 => (missing, 0),
        2 => (missing / 2, missing - (missing / 2)),
        _ => (0, missing),
    };

    let mut result = Vec::with_capacity(length);
    append_repeated_pad(&mut result, pad_string, left);
    result.extend_from_slice(bytes);
    let current_len = result.len();
    append_repeated_pad(&mut result, pad_string, current_len + right);
    result
}

fn append_repeated_pad(result: &mut Vec<u8>, pad_string: &[u8], target_len: usize) {
    while result.len() < target_len {
        let remaining = target_len - result.len();
        if remaining >= pad_string.len() {
            result.extend_from_slice(pad_string);
        } else {
            result.extend_from_slice(&pad_string[..remaining]);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_substr(value: EchoValue, offset: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(offset) = offset.int_value() else {
        return EchoValue::error();
    };

    let len = bytes.len() as i64;
    let start = if offset >= 0 { offset } else { len + offset }.clamp(0, len);

    echo_runtime_string(bytes[start as usize..].to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_substr_replace(
    value: EchoValue,
    replacement: EchoValue,
    offset: EchoValue,
    length: EchoValue,
) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(replacement) = replacement.string_bytes() else {
        return EchoValue::error();
    };
    let Some(offset) = offset.int_value() else {
        return EchoValue::error();
    };

    let len = bytes.len() as i64;
    let start = if offset >= 0 { offset } else { len + offset }.clamp(0, len);
    let end = if length.is_null() {
        len
    } else {
        let Some(length) = length.int_value() else {
            return EchoValue::error();
        };
        if length >= 0 {
            start.saturating_add(length).min(len)
        } else {
            (len + length).clamp(start, len)
        }
    };

    let start = start as usize;
    let end = end as usize;
    let mut result = Vec::with_capacity(bytes.len() - (end - start) + replacement.len());
    result.extend_from_slice(&bytes[..start]);
    result.extend_from_slice(&replacement);
    result.extend_from_slice(&bytes[end..]);
    echo_runtime_string(result)
}
