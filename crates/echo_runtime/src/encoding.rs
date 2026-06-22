use crc32fast::Hasher as Crc32Hasher;
use md5_digest::{Digest as _, Md5};
use sha1::Sha1;

use crate::{EchoString, EchoValue, string::php_string_map_builtin};

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
pub extern "C" fn echo_php_rawurldecode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| percent_decode(bytes, false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_urldecode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| percent_decode(bytes, true))
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

pub(crate) fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
