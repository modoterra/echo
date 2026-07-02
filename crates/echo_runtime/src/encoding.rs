use crc32fast::Hasher as Crc32Hasher;
use md5_digest::{Digest as _, Md5};
use sha1::Sha1;

use crate::{
    EchoArray, EchoString, EchoValue, collections::EchoArrayKey, echo_runtime_string,
    echo_value_array_new, echo_value_array_set, string::php_string_map_builtin,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PercentEncodingMode {
    RawUrl,
    FormUrl,
}

pub(crate) fn lowercase_hex_bytes(bytes: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize]);
        encoded.push(HEX[(byte & 0x0f) as usize]);
    }
    encoded
}

pub(crate) fn decode_hex(bytes: &[u8]) -> Option<Vec<u8>> {
    if bytes.len() % 2 != 0 {
        return None;
    }

    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        decoded.push((high << 4) | low);
    }

    Some(decoded)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_bin2hex(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            lowercase_hex_bytes(&bytes),
        )))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hex2bin(value: EchoValue) -> EchoValue {
    match value.string_bytes().and_then(|bytes| decode_hex(&bytes)) {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes)))),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_utf8_encode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        let mut encoded = Vec::with_capacity(bytes.len());
        for byte in bytes {
            if *byte < 0x80 {
                encoded.push(*byte);
            } else {
                encoded.push(0xc0 | (*byte >> 6));
                encoded.push(0x80 | (*byte & 0x3f));
            }
        }
        encoded
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_utf8_decode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, utf8_decode_latin1_bytes)
}

fn utf8_decode_latin1_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        let first = bytes[index];
        if first < 0x80 {
            decoded.push(first);
            index += 1;
            continue;
        }

        let Some((codepoint, len)) = decode_utf8_codepoint(&bytes[index..]) else {
            decoded.push(b'?');
            index += 1;
            continue;
        };

        if codepoint <= 0xff {
            decoded.push(codepoint as u8);
        } else {
            decoded.push(b'?');
        }
        index += len;
    }

    decoded
}

fn decode_utf8_codepoint(bytes: &[u8]) -> Option<(u32, usize)> {
    let first = *bytes.first()?;
    let (mut codepoint, len, min) = match first {
        0xc2..=0xdf => ((first & 0x1f) as u32, 2, 0x80),
        0xe0..=0xef => ((first & 0x0f) as u32, 3, 0x800),
        0xf0..=0xf4 => ((first & 0x07) as u32, 4, 0x10000),
        _ => return None,
    };

    if bytes.len() < len {
        return None;
    }

    for continuation in &bytes[1..len] {
        if continuation & 0xc0 != 0x80 {
            return None;
        }
        codepoint = (codepoint << 6) | (continuation & 0x3f) as u32;
    }

    if codepoint < min || (0xd800..=0xdfff).contains(&codepoint) || codepoint > 0x10ffff {
        return None;
    }

    Some((codepoint, len))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_crc32(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => {
            let mut hasher = Crc32Hasher::new();
            hasher.update(&bytes);
            EchoValue::int(hasher.finalize() as i64)
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_md5(value: EchoValue, binary: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => {
            let digest = Md5::digest(&bytes);
            let bytes = if binary.bool_value().unwrap_or(false) {
                digest.to_vec()
            } else {
                lowercase_hex_bytes(&digest)
            };
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sha1(value: EchoValue, binary: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => {
            let digest = Sha1::digest(&bytes);
            let bytes = if binary.bool_value().unwrap_or(false) {
                digest.to_vec()
            } else {
                lowercase_hex_bytes(&digest)
            };
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

pub(crate) fn encode_base64(bytes: &[u8]) -> Vec<u8> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = Vec::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);

        encoded.push(TABLE[(first >> 2) as usize]);
        encoded.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize]);

        if chunk.len() > 1 {
            encoded.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize]);
        } else {
            encoded.push(b'=');
        }

        if chunk.len() > 2 {
            encoded.push(TABLE[(third & 0b0011_1111) as usize]);
        } else {
            encoded.push(b'=');
        }
    }

    encoded
}

pub(crate) fn decode_base64_non_strict(bytes: &[u8]) -> Vec<u8> {
    let mut values = Vec::new();
    for byte in bytes.iter().copied() {
        match base64_value(byte) {
            Some(value) => values.push(value),
            None if byte == b'=' => values.push(64),
            None => {}
        }
    }

    let mut decoded = Vec::with_capacity(values.len() / 4 * 3);
    for chunk in values.chunks(4) {
        if chunk.len() < 2 {
            break;
        }

        let first = chunk[0];
        let second = chunk[1];
        if first >= 64 || second >= 64 {
            break;
        }

        decoded.push((first << 2) | (second >> 4));

        let Some(&third) = chunk.get(2) else {
            break;
        };
        if third >= 64 {
            break;
        }
        decoded.push(((second & 0b0000_1111) << 4) | (third >> 2));

        let Some(&fourth) = chunk.get(3) else {
            break;
        };
        if fourth >= 64 {
            break;
        }
        decoded.push(((third & 0b0000_0011) << 6) | fourth);
    }

    decoded
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_base64_encode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, encode_base64)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_convert_uuencode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, uuencode_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_convert_uudecode(value: EchoValue) -> EchoValue {
    match value
        .string_bytes()
        .and_then(|bytes| uudecode_bytes(&bytes))
    {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes)))),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_base64_decode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, decode_base64_non_strict)
}

pub(crate) fn percent_encode(bytes: &[u8], mode: PercentEncodingMode) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = Vec::with_capacity(bytes.len());
    for byte in bytes {
        if percent_encode_keeps_byte(*byte, mode) {
            encoded.push(*byte);
        } else if mode == PercentEncodingMode::FormUrl && *byte == b' ' {
            encoded.push(b'+');
        } else {
            encoded.push(b'%');
            encoded.push(HEX[(byte >> 4) as usize]);
            encoded.push(HEX[(byte & 0x0f) as usize]);
        }
    }

    encoded
}

fn http_build_query_array(
    data: &EchoArray,
    numeric_prefix: &[u8],
    separator: &[u8],
    encoding_mode: PercentEncodingMode,
) -> Vec<u8> {
    let mut pairs = Vec::new();
    for (key, value) in data.keys.iter().zip(&data.values) {
        if value.is_null() {
            continue;
        }
        let key = http_query_key_bytes(key, Some(numeric_prefix));
        append_http_query_pairs(&mut pairs, key, *value, encoding_mode);
    }
    pairs.join(separator)
}

fn append_http_query_pairs(
    pairs: &mut Vec<Vec<u8>>,
    key: Vec<u8>,
    value: EchoValue,
    encoding_mode: PercentEncodingMode,
) {
    if value.is_null() {
        return;
    }

    if value.is_array() {
        let Some(array) = (unsafe { (value.payload as *const EchoArray).as_ref() }) else {
            return;
        };
        for (child_key, child_value) in array.keys.iter().zip(&array.values) {
            let child_key = http_query_nested_key_bytes(&key, child_key);
            append_http_query_pairs(pairs, child_key, *child_value, encoding_mode);
        }
        return;
    }

    let Some(value) = value.string_bytes() else {
        return;
    };
    let mut pair = percent_encode(&key, encoding_mode);
    pair.push(b'=');
    pair.extend(percent_encode(&value, encoding_mode));
    pairs.push(pair);
}

fn http_query_key_bytes(key: &EchoArrayKey, numeric_prefix: Option<&[u8]>) -> Vec<u8> {
    match key {
        EchoArrayKey::Int(value) => {
            let mut bytes = numeric_prefix.unwrap_or_default().to_vec();
            bytes.extend(value.to_string().as_bytes());
            bytes
        }
        EchoArrayKey::String(bytes) => bytes.clone(),
    }
}

fn http_query_nested_key_bytes(parent: &[u8], key: &EchoArrayKey) -> Vec<u8> {
    let mut bytes = parent.to_vec();
    bytes.push(b'[');
    bytes.extend(http_query_key_bytes(key, None));
    bytes.push(b']');
    bytes
}

pub(crate) fn percent_decode(bytes: &[u8], decode_plus: bool) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if decode_plus && bytes[index] == b'+' {
            decoded.push(b' ');
            index += 1;
            continue;
        }

        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_nibble(bytes[index + 1]), hex_nibble(bytes[index + 2]))
            {
                decoded.push((high << 4) | low);
                index += 3;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    decoded
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rawurlencode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        percent_encode(bytes, PercentEncodingMode::RawUrl)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_urlencode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        percent_encode(bytes, PercentEncodingMode::FormUrl)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_http_build_query(
    data: EchoValue,
    numeric_prefix: EchoValue,
    arg_separator: EchoValue,
    encoding_type: EchoValue,
) -> EchoValue {
    if !data.is_array() {
        return EchoValue::error();
    }
    let Some(data) = (unsafe { (data.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(numeric_prefix) = numeric_prefix.string_bytes() else {
        return EchoValue::error();
    };
    let separator = if arg_separator.is_null() {
        b"&".to_vec()
    } else {
        arg_separator
            .string_bytes()
            .unwrap_or_else(|| b"&".to_vec())
    };
    let encoding_mode = match encoding_type.php_int_value().unwrap_or(1) {
        2 => PercentEncodingMode::RawUrl,
        _ => PercentEncodingMode::FormUrl,
    };

    echo_runtime_string(http_build_query_array(
        data,
        &numeric_prefix,
        &separator,
        encoding_mode,
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rawurldecode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| percent_decode(bytes, false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_urldecode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| percent_decode(bytes, true))
}

#[derive(Debug, Default)]
struct ParsedUrl {
    scheme: Option<Vec<u8>>,
    host: Option<Vec<u8>>,
    port: Option<i64>,
    user: Option<Vec<u8>>,
    pass: Option<Vec<u8>>,
    path: Option<Vec<u8>>,
    query: Option<Vec<u8>>,
    fragment: Option<Vec<u8>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_parse_url(url: EchoValue) -> EchoValue {
    let Some(bytes) = url.string_bytes() else {
        return EchoValue::bool(false);
    };

    let Some(parsed) = parse_url_parts(&bytes) else {
        return EchoValue::bool(false);
    };

    parsed_url_array(parsed)
}

fn parsed_url_array(parsed: ParsedUrl) -> EchoValue {
    let mut result = echo_value_array_new();
    if let Some(value) = parsed.scheme {
        result = parsed_url_string_part(result, b"scheme", value);
    }
    if let Some(value) = parsed.host {
        result = parsed_url_string_part(result, b"host", value);
    }
    if let Some(value) = parsed.port {
        result = echo_value_array_set(
            result,
            echo_runtime_string(b"port".to_vec()),
            EchoValue::int(value),
        );
    }
    if let Some(value) = parsed.user {
        result = parsed_url_string_part(result, b"user", value);
    }
    if let Some(value) = parsed.pass {
        result = parsed_url_string_part(result, b"pass", value);
    }
    if let Some(value) = parsed.path {
        result = parsed_url_string_part(result, b"path", value);
    }
    if let Some(value) = parsed.query {
        result = parsed_url_string_part(result, b"query", value);
    }
    if let Some(value) = parsed.fragment {
        result = parsed_url_string_part(result, b"fragment", value);
    }
    result
}

fn parsed_url_string_part(array: EchoValue, key: &[u8], value: Vec<u8>) -> EchoValue {
    echo_value_array_set(
        array,
        echo_runtime_string(key.to_vec()),
        echo_runtime_string(value),
    )
}

fn parse_url_parts(bytes: &[u8]) -> Option<ParsedUrl> {
    if bytes.is_empty() {
        return Some(ParsedUrl {
            path: Some(Vec::new()),
            ..ParsedUrl::default()
        });
    }

    let mut parsed = ParsedUrl::default();
    let mut rest = bytes;

    if let Some(colon) = first_url_scheme_colon(rest) {
        parsed.scheme = Some(rest[..colon].to_vec());
        rest = &rest[colon + 1..];
    }

    if rest.starts_with(b"//") {
        rest = &rest[2..];
        let authority_end = rest
            .iter()
            .position(|byte| matches!(*byte, b'/' | b'?' | b'#'))
            .unwrap_or(rest.len());
        parse_url_authority(&mut parsed, &rest[..authority_end])?;
        rest = &rest[authority_end..];
    }

    let path_end = rest
        .iter()
        .position(|byte| matches!(*byte, b'?' | b'#'))
        .unwrap_or(rest.len());
    if path_end > 0 {
        parsed.path = Some(rest[..path_end].to_vec());
    }
    rest = &rest[path_end..];

    if rest.starts_with(b"?") {
        rest = &rest[1..];
        let query_end = rest
            .iter()
            .position(|byte| *byte == b'#')
            .unwrap_or(rest.len());
        parsed.query = Some(rest[..query_end].to_vec());
        rest = &rest[query_end..];
    }

    if rest.starts_with(b"#") {
        parsed.fragment = Some(rest[1..].to_vec());
    }

    Some(parsed)
}

fn first_url_scheme_colon(bytes: &[u8]) -> Option<usize> {
    let colon = bytes.iter().position(|byte| *byte == b':')?;
    if colon == 0
        || bytes[..colon]
            .iter()
            .any(|byte| matches!(*byte, b'/' | b'?' | b'#'))
    {
        return None;
    }
    Some(colon)
}

fn parse_url_authority(parsed: &mut ParsedUrl, authority: &[u8]) -> Option<()> {
    let host_port = match authority.iter().rposition(|byte| *byte == b'@') {
        Some(at) => {
            let userinfo = &authority[..at];
            match userinfo.iter().position(|byte| *byte == b':') {
                Some(colon) => {
                    parsed.user = Some(userinfo[..colon].to_vec());
                    parsed.pass = Some(userinfo[colon + 1..].to_vec());
                }
                None => parsed.user = Some(userinfo.to_vec()),
            }
            &authority[at + 1..]
        }
        None => authority,
    };

    if host_port.is_empty() {
        return Some(());
    }

    if host_port.starts_with(b"[") {
        let end = host_port.iter().position(|byte| *byte == b']')?;
        parsed.host = Some(host_port[..=end].to_vec());
        if matches!(host_port.get(end + 1), Some(b':')) {
            parsed.port = parse_url_port(&host_port[end + 2..]);
        }
        return Some(());
    }

    match host_port.iter().rposition(|byte| *byte == b':') {
        Some(colon) if host_port[colon + 1..].iter().all(u8::is_ascii_digit) => {
            parsed.host = Some(host_port[..colon].to_vec());
            parsed.port = parse_url_port(&host_port[colon + 1..]);
        }
        _ => parsed.host = Some(host_port.to_vec()),
    }

    Some(())
}

fn parse_url_port(bytes: &[u8]) -> Option<i64> {
    if bytes.is_empty() {
        return None;
    }
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_escapeshellarg(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, escape_shell_arg_unix)
}

fn escape_shell_arg_unix(bytes: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(bytes.len() + 2);
    escaped.push(b'\'');
    for byte in bytes {
        if *byte == b'\'' {
            escaped.extend_from_slice(b"'\\''");
        } else {
            escaped.push(*byte);
        }
    }
    escaped.push(b'\'');
    escaped
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_escapeshellcmd(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, escape_shell_cmd_unix)
}

fn escape_shell_cmd_unix(bytes: &[u8]) -> Vec<u8> {
    let single_quotes_unpaired = bytes.iter().filter(|byte| **byte == b'\'').count() % 2 == 1;
    let double_quotes_unpaired = bytes.iter().filter(|byte| **byte == b'"').count() % 2 == 1;
    let mut escaped = Vec::with_capacity(bytes.len());

    for byte in bytes {
        if shell_cmd_byte_needs_escape(*byte)
            || (*byte == b'\'' && single_quotes_unpaired)
            || (*byte == b'"' && double_quotes_unpaired)
        {
            escaped.push(b'\\');
        }
        escaped.push(*byte);
    }

    escaped
}

fn shell_cmd_byte_needs_escape(byte: u8) -> bool {
    matches!(
        byte,
        b'#' | b'&'
            | b';'
            | b'`'
            | b'|'
            | b'*'
            | b'?'
            | b'~'
            | b'<'
            | b'>'
            | b'^'
            | b'('
            | b')'
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'$'
            | b'\\'
            | b'\n'
    )
}

pub(crate) fn quoted_printable_encode_bytes(bytes: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = Vec::with_capacity(bytes.len());
    for byte in bytes.iter().copied() {
        if matches!(byte, b'!'..=b'<' | b'>'..=b'~' | b' ' | b'\t') {
            encoded.push(byte);
        } else {
            encoded.push(b'=');
            encoded.push(HEX[(byte >> 4) as usize]);
            encoded.push(HEX[(byte & 0x0f) as usize]);
        }
    }
    encoded
}

pub(crate) fn quoted_printable_decode_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'=' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }

        if bytes.get(index + 1) == Some(&b'\r') && bytes.get(index + 2) == Some(&b'\n') {
            index += 3;
            continue;
        }
        if bytes.get(index + 1) == Some(&b'\n') {
            index += 2;
            continue;
        }

        if index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_nibble(bytes[index + 1]), hex_nibble(bytes[index + 2]))
            {
                decoded.push((high << 4) | low);
                index += 3;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    decoded
}

fn percent_encode_keeps_byte(byte: u8, mode: PercentEncodingMode) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(byte, b'-' | b'_' | b'.')
        || (mode == PercentEncodingMode::RawUrl && byte == b'~')
}

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn uuencode_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();

    for chunk in bytes.chunks(45) {
        encoded.push(uuencode_byte(chunk.len() as u8));
        for triple in chunk.chunks(3) {
            let first = triple[0];
            let second = *triple.get(1).unwrap_or(&0);
            let third = *triple.get(2).unwrap_or(&0);

            encoded.push(uuencode_byte(first >> 2));
            encoded.push(uuencode_byte(((first << 4) | (second >> 4)) & 0x3f));
            encoded.push(uuencode_byte(((second << 2) | (third >> 6)) & 0x3f));
            encoded.push(uuencode_byte(third & 0x3f));
        }
        encoded.push(b'\n');
    }

    encoded.extend_from_slice(b"`\n");
    encoded
}

fn uuencode_byte(value: u8) -> u8 {
    let value = (value & 0x3f) + 0x20;
    if value == b' ' { b'`' } else { value }
}

fn uudecode_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::new();
    let mut saw_terminator = false;

    for raw_line in bytes.split(|byte| *byte == b'\n') {
        let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);
        if line.is_empty() {
            continue;
        }

        let expected_len = uudecode_byte(line[0])? as usize;
        if expected_len == 0 {
            saw_terminator = true;
            break;
        }

        let encoded = &line[1..];
        let needed_groups = expected_len.div_ceil(3);
        if encoded.len() < needed_groups * 4 {
            return None;
        }

        let mut line_bytes = Vec::with_capacity(needed_groups * 3);
        for group in encoded[..needed_groups * 4].chunks_exact(4) {
            let first = uudecode_byte(group[0])?;
            let second = uudecode_byte(group[1])?;
            let third = uudecode_byte(group[2])?;
            let fourth = uudecode_byte(group[3])?;

            line_bytes.push((first << 2) | (second >> 4));
            line_bytes.push((second << 4) | (third >> 2));
            line_bytes.push((third << 6) | fourth);
        }

        if line_bytes.len() < expected_len {
            return None;
        }
        decoded.extend_from_slice(&line_bytes[..expected_len]);
    }

    saw_terminator.then_some(decoded)
}

fn uudecode_byte(byte: u8) -> Option<u8> {
    match byte {
        b'`' => Some(0),
        0x20..=0x5f => Some((byte - 0x20) & 0x3f),
        _ => None,
    }
}

pub(crate) fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
