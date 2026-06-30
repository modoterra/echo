use bcrypt::hash_with_salt;
use crypt3_rs::crypt::{bsdi, md5, sha256, sha512, unix};
use rand::{RngCore, rngs::OsRng};

use crate::encoding::lowercase_hex_bytes;
use crate::{
    EchoObject, EchoValue,
    collections::{EchoArray, EchoArrayKey},
    echo_runtime_string, echo_value_array_append, echo_value_array_new, echo_value_array_set,
    echo_value_object_new,
};
use md5_digest::{Digest as Md5Digest, Md5};
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::Digest as Sha2Digest;
use sha2::{Sha224, Sha256, Sha384, Sha512, Sha512_224, Sha512_256};
use sha3::Digest as Sha3Digest;
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use std::fs;
use std::io::Read;

const BCRYPT_ALGO_COST_MIN: u32 = 4;
const BCRYPT_ALGO_COST_MAX: u32 = 31;
const BCRYPT_DEFAULT_COST: u32 = 10;
const BCRYPT_COST_LEN: usize = 2;
const BCRYPT_PREFIX_LEN: usize = 4;
const BCRYPT_PREFIXES: [&[u8]; 4] = [b"$2y$", b"$2a$", b"$2x$", b"$2b$"];
const BCRYPT_SALT_BYTES: usize = 16;
const BCRYPT_SALT_CHARS: usize = 22;
const PASSWORD_BCRYPT_ID: i64 = 1;
const PASSWORD_UNKNOWN_ID: i64 = 0;
const BCRYPT_BASE64: &[u8; 64] =
    b"./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const HASH_HMAC_OPTION: i64 = 1;
const HASH_CTX_ALGORITHM_FIELD: &str = "__echo_hash_ctx_algo";
const HASH_CTX_DATA_FIELD: &str = "__echo_hash_ctx_data";
const HASH_CTX_FINALIZED_FIELD: &str = "__echo_hash_ctx_finalized";
const HASH_CTX_HMAC_FIELD: &str = "__echo_hash_ctx_hmac";
const HASH_CTX_KEY_FIELD: &str = "__echo_hash_ctx_key";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PasswordAlgorithm {
    Bcrypt,
}

#[derive(Debug, Clone, Copy)]
enum NonBcryptCryptAlgorithm {
    StdDes,
    ExtDes,
    Md5,
    Sha256,
    Sha512,
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_random_bytes(length: EchoValue) -> EchoValue {
    let Some(length) = length.php_int_value() else {
        return EchoValue::bool(false);
    };
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::bool(false);
    };
    if length <= 0 {
        return EchoValue::bool(false);
    }

    let mut bytes = vec![0_u8; length];
    if !fill_random_bytes(&mut bytes) {
        return EchoValue::bool(false);
    }
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_random_int(min_value: EchoValue, max_value: EchoValue) -> EchoValue {
    let Some(min_value) = min_value.php_int_value() else {
        return EchoValue::bool(false);
    };
    let Some(max_value) = max_value.php_int_value() else {
        return EchoValue::bool(false);
    };

    match random_int_range(min_value, max_value) {
        Some(value) => EchoValue::int(value),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_crypt(password: EchoValue, salt: EchoValue) -> EchoValue {
    let Some(password) = password.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(salt) = salt.string_bytes() else {
        return EchoValue::bool(false);
    };

    if let Some((version, cost, salt)) = parse_crypt_bcrypt_salt(&salt) {
        if let Some(hash) = bcrypt_hash_with_salt_and_cost(&password, cost, version, salt) {
            return echo_runtime_string(hash);
        }
        return EchoValue::bool(false);
    }

    match parse_non_bcrypt_crypt_algorithm(&salt) {
        Some(NonBcryptCryptAlgorithm::StdDes) => match non_bcrypt_crypt_std_des(&password, &salt) {
            Some(hash) => echo_runtime_string(hash),
            None => EchoValue::bool(false),
        },
        Some(NonBcryptCryptAlgorithm::ExtDes) => match non_bcrypt_crypt_ext_des(&password, &salt) {
            Some(hash) => echo_runtime_string(hash),
            None => EchoValue::bool(false),
        },
        Some(NonBcryptCryptAlgorithm::Md5) => match non_bcrypt_crypt_md5(&password, &salt) {
            Some(hash) => echo_runtime_string(hash),
            None => EchoValue::bool(false),
        },
        Some(NonBcryptCryptAlgorithm::Sha256) => match non_bcrypt_crypt_sha256(&password, &salt) {
            Some(hash) => echo_runtime_string(hash),
            None => EchoValue::bool(false),
        },
        Some(NonBcryptCryptAlgorithm::Sha512) => match non_bcrypt_crypt_sha512(&password, &salt) {
            Some(hash) => echo_runtime_string(hash),
            None => EchoValue::bool(false),
        },
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_password_algos() -> EchoValue {
    let result = echo_value_array_new();
    echo_value_array_append(result, echo_runtime_string(b"2y".to_vec()))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_password_get_info(hash: EchoValue) -> EchoValue {
    let Some(hash) = hash.string_bytes() else {
        return EchoValue::error();
    };
    let mut info = echo_value_array_new();

    if let Some(cost) = parse_crypt_bcrypt_cost(&hash) {
        let mut options = echo_value_array_new();
        options = echo_value_array_set(
            options,
            echo_runtime_string(b"cost".to_vec()),
            EchoValue::int(cost as i64),
        );
        info = echo_value_array_set(
            info,
            echo_runtime_string(b"algo".to_vec()),
            EchoValue::int(PASSWORD_BCRYPT_ID),
        );
        info = echo_value_array_set(
            info,
            echo_runtime_string(b"algoName".to_vec()),
            echo_runtime_string(b"2y".to_vec()),
        );
        info = echo_value_array_set(info, echo_runtime_string(b"options".to_vec()), options);
        return info;
    }

    info = echo_value_array_set(
        info,
        echo_runtime_string(b"algo".to_vec()),
        EchoValue::int(PASSWORD_UNKNOWN_ID),
    );
    info = echo_value_array_set(
        info,
        echo_runtime_string(b"algoName".to_vec()),
        echo_runtime_string(b"unknown".to_vec()),
    );
    echo_value_array_set(
        info,
        echo_runtime_string(b"options".to_vec()),
        echo_value_array_new(),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_password_hash(
    password: EchoValue,
    algorithm: EchoValue,
    options: EchoValue,
) -> EchoValue {
    let Some(password) = password.string_bytes() else {
        return EchoValue::bool(false);
    };

    if parse_password_algorithm(algorithm) != Some(PasswordAlgorithm::Bcrypt) {
        return EchoValue::bool(false);
    }
    let Some(cost) = parse_bcrypt_cost_option(options, BCRYPT_DEFAULT_COST) else {
        return EchoValue::bool(false);
    };
    let Some(hash) = bcrypt_hash_with_random_salt(&password, cost) else {
        return EchoValue::bool(false);
    };
    echo_runtime_string(hash)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_password_needs_rehash(
    hash: EchoValue,
    algorithm: EchoValue,
    options: EchoValue,
) -> EchoValue {
    let requested_cost = if parse_password_algorithm(algorithm) == Some(PasswordAlgorithm::Bcrypt) {
        parse_bcrypt_cost_option(options, BCRYPT_DEFAULT_COST)
    } else {
        return EchoValue::bool(false);
    };
    let Some(requested_cost) = requested_cost else {
        return EchoValue::bool(false);
    };

    let Some(hash_bytes) = hash.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(current_cost) = parse_crypt_bcrypt_cost(&hash_bytes) else {
        return EchoValue::bool(true);
    };

    EchoValue::bool(current_cost != requested_cost)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_password_verify(password: EchoValue, hash: EchoValue) -> EchoValue {
    let Some(password) = password.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(hash_bytes) = hash.string_bytes() else {
        return EchoValue::bool(false);
    };

    let Some(recomputed) = bcrypt_crypt_with_hash_salt(&password, &hash_bytes) else {
        return EchoValue::bool(false);
    };
    EchoValue::bool(constant_time_equals(&recomputed, &hash_bytes))
}

fn parse_password_algorithm(value: EchoValue) -> Option<PasswordAlgorithm> {
    if value.is_null() {
        return Some(PasswordAlgorithm::Bcrypt);
    }

    if let Some(value) = value.int_value() {
        if value == PASSWORD_BCRYPT_ID {
            return Some(PasswordAlgorithm::Bcrypt);
        }
        return None;
    }

    let Some(bytes) = value.string_bytes() else {
        return None;
    };
    let mut normalized = bytes;
    for byte in &mut normalized {
        byte.make_ascii_lowercase();
    }
    match normalized.as_slice() {
        b"bcrypt" | b"2y" | b"2a" | b"2b" => Some(PasswordAlgorithm::Bcrypt),
        _ => None,
    }
}

fn parse_bcrypt_cost_option(options: EchoValue, default_cost: u32) -> Option<u32> {
    if options.is_null() {
        return Some(default_cost);
    }
    if !options.is_array() {
        return None;
    }

    let Some(options) = (unsafe { (options.payload as *const EchoArray).as_ref() }) else {
        return None;
    };

    let mut cost = default_cost;
    for (key, value) in options.keys.iter().zip(&options.values) {
        if let EchoArrayKey::String(key) = key {
            if key.as_slice() != b"cost" {
                continue;
            }
            let Some(candidate) = value.php_int_value() else {
                return None;
            };
            if !(BCRYPT_ALGO_COST_MIN as i64..=BCRYPT_ALGO_COST_MAX as i64).contains(&candidate) {
                return None;
            }
            cost = candidate as u32;
        }
    }

    Some(cost)
}

fn parse_crypt_bcrypt_salt(salt: &[u8]) -> Option<(u8, u32, &[u8])> {
    let min_len = BCRYPT_PREFIX_LEN + BCRYPT_COST_LEN + 1 + BCRYPT_SALT_CHARS;
    if salt.len() < min_len {
        return None;
    }
    if !BCRYPT_PREFIXES
        .iter()
        .any(|prefix| salt.starts_with(prefix))
    {
        return None;
    }
    if salt.get(BCRYPT_PREFIX_LEN + BCRYPT_COST_LEN) != Some(&b'$') {
        return None;
    }
    let Some(cost) =
        parse_two_digits(&salt[BCRYPT_PREFIX_LEN..BCRYPT_PREFIX_LEN + BCRYPT_COST_LEN])
    else {
        return None;
    };
    let version = *salt.get(2)?;
    let salt = salt
        .get(BCRYPT_PREFIX_LEN + BCRYPT_COST_LEN + 1..)?
        .get(..BCRYPT_SALT_CHARS)?;
    if !salt.iter().all(|byte| is_bcrypt_base64_char(*byte)) {
        return None;
    }

    Some((version, cost, salt))
}

fn parse_crypt_bcrypt_cost(hash: &[u8]) -> Option<u32> {
    let min_len = BCRYPT_PREFIX_LEN + BCRYPT_COST_LEN + 1 + BCRYPT_SALT_CHARS;
    if hash.len() < min_len {
        return None;
    }
    if !BCRYPT_PREFIXES
        .iter()
        .any(|prefix| hash.starts_with(prefix))
    {
        return None;
    }
    if hash.get(BCRYPT_PREFIX_LEN + BCRYPT_COST_LEN) != Some(&b'$') {
        return None;
    }
    let cost = parse_two_digits(&hash[BCRYPT_PREFIX_LEN..BCRYPT_PREFIX_LEN + BCRYPT_COST_LEN])?;
    Some(cost)
}

fn parse_two_digits(bytes: &[u8]) -> Option<u32> {
    if bytes.len() != BCRYPT_COST_LEN {
        return None;
    }
    if !bytes[0].is_ascii_digit() || !bytes[1].is_ascii_digit() {
        return None;
    }
    let cost = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
    if !(BCRYPT_ALGO_COST_MIN..=BCRYPT_ALGO_COST_MAX).contains(&u32::from(cost)) {
        None
    } else {
        Some(u32::from(cost))
    }
}

fn bcrypt_hash_with_random_salt(password: &[u8], cost: u32) -> Option<Vec<u8>> {
    if !(BCRYPT_ALGO_COST_MIN..=BCRYPT_ALGO_COST_MAX).contains(&cost) {
        return None;
    }

    let salt = random_bcrypt_salt()?;
    bcrypt_hash_with_salt_and_cost(password, cost, b'y', salt.as_bytes())
}

fn bcrypt_hash_with_salt_and_cost(
    password: &[u8],
    cost: u32,
    version: u8,
    salt: &[u8],
) -> Option<Vec<u8>> {
    if cost < BCRYPT_ALGO_COST_MIN || cost > BCRYPT_ALGO_COST_MAX {
        return None;
    }
    if salt.len() != BCRYPT_SALT_CHARS || !salt.iter().all(|byte| is_bcrypt_base64_char(*byte)) {
        return None;
    }

    let salt = bcrypt_decode_salt_bytes(salt)?;
    let mut hash = hash_with_salt(password, cost, salt)
        .ok()?
        .to_string()
        .into_bytes();
    if hash.len() > 2 {
        hash[2] = version;
    }
    Some(hash)
}

fn parse_non_bcrypt_crypt_algorithm(salt: &[u8]) -> Option<NonBcryptCryptAlgorithm> {
    if salt.starts_with(b"_") {
        return Some(NonBcryptCryptAlgorithm::ExtDes);
    }
    if salt.len() >= 2 && !salt.starts_with(b"$") {
        return Some(NonBcryptCryptAlgorithm::StdDes);
    }
    if salt.starts_with(b"$1$") {
        return Some(NonBcryptCryptAlgorithm::Md5);
    }
    if salt.starts_with(b"$5$") {
        return Some(NonBcryptCryptAlgorithm::Sha256);
    }
    if salt.starts_with(b"$6$") {
        return Some(NonBcryptCryptAlgorithm::Sha512);
    }
    None
}

#[allow(deprecated)]
fn non_bcrypt_crypt_std_des(password: &[u8], salt: &[u8]) -> Option<Vec<u8>> {
    let salt = std::str::from_utf8(salt).ok()?;
    unix::hash_with(salt, password)
        .ok()
        .map(|hash| hash.to_string().into_bytes())
}

#[allow(deprecated)]
fn non_bcrypt_crypt_ext_des(password: &[u8], salt: &[u8]) -> Option<Vec<u8>> {
    let salt = std::str::from_utf8(salt).ok()?;
    bsdi::hash_with(salt, password)
        .ok()
        .map(|hash| hash.to_string().into_bytes())
}

#[allow(deprecated)]
fn non_bcrypt_crypt_md5(password: &[u8], salt: &[u8]) -> Option<Vec<u8>> {
    let salt = std::str::from_utf8(salt).ok()?;
    md5::hash_with(salt, password)
        .ok()
        .map(|hash| hash.to_string().into_bytes())
}

#[allow(deprecated)]
fn non_bcrypt_crypt_sha256(password: &[u8], salt: &[u8]) -> Option<Vec<u8>> {
    let salt = std::str::from_utf8(salt).ok()?;
    sha256::hash_with(salt, password)
        .ok()
        .map(|hash| hash.to_string().into_bytes())
}

fn non_bcrypt_crypt_sha512(password: &[u8], salt: &[u8]) -> Option<Vec<u8>> {
    let salt = std::str::from_utf8(salt).ok()?;
    sha512::hash_with(salt, password)
        .ok()
        .map(|hash| hash.to_string().into_bytes())
}

fn bcrypt_crypt_with_hash_salt(password: &[u8], hash: &[u8]) -> Option<Vec<u8>> {
    let (version, cost, salt) = parse_crypt_bcrypt_salt(hash)?;
    bcrypt_hash_with_salt_and_cost(password, cost, version, salt)
}

fn random_bcrypt_salt() -> Option<String> {
    let mut bytes = [0_u8; BCRYPT_SALT_BYTES];
    if !fill_random_bytes(&mut bytes) {
        return None;
    }
    Some(encode_bcrypt_base64(&bytes, BCRYPT_SALT_CHARS))
}

fn encode_bcrypt_base64(bytes: &[u8], length: usize) -> String {
    let mut output = Vec::with_capacity(length);
    let mut index = 0usize;

    while output.len() < length {
        let left = bytes.get(index).copied().unwrap_or(0) as u32;
        let middle = bytes.get(index + 1).copied().unwrap_or(0) as u32;
        let right = bytes.get(index + 2).copied().unwrap_or(0) as u32;

        output.push(BCRYPT_BASE64[((left >> 2) & 0x3f) as usize]);
        output.push(BCRYPT_BASE64[(((left << 4) | (middle >> 4)) & 0x3f) as usize]);
        if output.len() >= length {
            break;
        }
        output.push(BCRYPT_BASE64[(((middle << 2) | (right >> 6)) & 0x3f) as usize]);
        if output.len() >= length {
            break;
        }
        output.push(BCRYPT_BASE64[(right & 0x3f) as usize]);
        index += 3;
    }

    output.truncate(length);
    String::from_utf8(output).expect("bcrypt base64 output")
}

fn is_bcrypt_base64_char(byte: u8) -> bool {
    BCRYPT_BASE64.contains(&byte)
}

fn bcrypt_decode_salt_bytes(salt: &[u8]) -> Option<[u8; BCRYPT_SALT_BYTES]> {
    if salt.len() != BCRYPT_SALT_CHARS || !salt.iter().all(|byte| is_bcrypt_base64_char(*byte)) {
        return None;
    }

    let mut decoded = [0_u8; BCRYPT_SALT_BYTES];
    let mut output_index = 0usize;
    let mut bit_buffer = 0_u32;
    let mut bit_count = 0_u8;

    for byte in salt.iter().copied() {
        let value = BCRYPT_BASE64.iter().position(|value| *value == byte)? as u32;
        bit_buffer = (bit_buffer << 6) | value;
        bit_count = bit_count.saturating_add(6);

        while bit_count >= 8 {
            bit_count -= 8;
            let shift = u32::from(bit_count);
            if output_index >= BCRYPT_SALT_BYTES {
                return Some(decoded);
            }
            decoded[output_index] = ((bit_buffer >> shift) & 0xff) as u8;
            output_index += 1;
        }
        if output_index == BCRYPT_SALT_BYTES {
            break;
        }
    }

    if output_index == BCRYPT_SALT_BYTES {
        Some(decoded)
    } else {
        None
    }
}

fn random_int_range(min: i64, max: i64) -> Option<i64> {
    if min > max {
        return None;
    }

    let span = i128::from(max)
        .checked_sub(i128::from(min))
        .and_then(|value| value.checked_add(1))?;
    if span <= 0 {
        return None;
    }

    let span = span as u128;
    let max_u128 = u64::MAX as u128 + 1;
    if span == max_u128 {
        let offset = random_u64()?;
        return i64::try_from(i128::from(min) + i128::from(offset)).ok();
    }

    let remainder = max_u128 % span;
    let limit = max_u128 - remainder;
    let sample = loop {
        let candidate = random_u64()?;
        if u128::from(candidate) < limit {
            break candidate as u128;
        }
    };
    let offset = sample % span;
    i64::try_from(i128::from(min) + i128::try_from(offset).ok()?).ok()
}

fn random_u64() -> Option<u64> {
    let mut bytes = [0_u8; 8];
    if !fill_random_bytes(&mut bytes) {
        return None;
    }
    Some(u64::from_le_bytes(bytes))
}

fn fill_random_bytes(bytes: &mut [u8]) -> bool {
    OsRng.try_fill_bytes(bytes).is_ok()
}

fn constant_time_equals(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (l, r) in left.iter().zip(right.iter()) {
        difference |= l ^ r;
    }
    difference == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HashAlgorithm {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha512_224,
    Sha512_256,
    Sha3_224,
    Sha3_256,
    Sha3_384,
    Sha3_512,
    Crc32,
    Crc32b,
    Adler32,
}

#[derive(Debug, Clone, Copy)]
struct HashAlgorithmDef {
    name: &'static str,
    algorithm: HashAlgorithm,
    hmac_capable: bool,
}

const HASH_ALGORITHMS: &[HashAlgorithmDef] = &[
    HashAlgorithmDef {
        name: "md5",
        algorithm: HashAlgorithm::Md5,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha1",
        algorithm: HashAlgorithm::Sha1,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha224",
        algorithm: HashAlgorithm::Sha224,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha256",
        algorithm: HashAlgorithm::Sha256,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha384",
        algorithm: HashAlgorithm::Sha384,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha512",
        algorithm: HashAlgorithm::Sha512,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha512/224",
        algorithm: HashAlgorithm::Sha512_224,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha512/256",
        algorithm: HashAlgorithm::Sha512_256,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha3-224",
        algorithm: HashAlgorithm::Sha3_224,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha3-256",
        algorithm: HashAlgorithm::Sha3_256,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha3-384",
        algorithm: HashAlgorithm::Sha3_384,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "sha3-512",
        algorithm: HashAlgorithm::Sha3_512,
        hmac_capable: true,
    },
    HashAlgorithmDef {
        name: "crc32",
        algorithm: HashAlgorithm::Crc32,
        hmac_capable: false,
    },
    HashAlgorithmDef {
        name: "crc32b",
        algorithm: HashAlgorithm::Crc32b,
        hmac_capable: false,
    },
    HashAlgorithmDef {
        name: "adler32",
        algorithm: HashAlgorithm::Adler32,
        hmac_capable: false,
    },
];

#[derive(Debug, Clone)]
struct HashContext {
    algorithm: Vec<u8>,
    is_hmac: bool,
    key: Vec<u8>,
    data: Vec<u8>,
    finalized: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_md5_file(
    filename: EchoValue,
    raw: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(filename) = file_bytes(filename) else {
        return EchoValue::bool(false);
    };
    hash_output(
        hash_value_algorithm("md5"),
        &filename,
        raw.bool_value().unwrap_or(false),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sha1_file(
    filename: EchoValue,
    raw: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(filename) = file_bytes(filename) else {
        return EchoValue::bool(false);
    };
    hash_output(
        hash_value_algorithm("sha1"),
        &filename,
        raw.bool_value().unwrap_or(false),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash(
    algorithm: EchoValue,
    data: EchoValue,
    raw_output: EchoValue,
) -> EchoValue {
    let Some(algorithm) = algorithm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(algorithm) = parse_hash_algorithm(&algorithm) else {
        return EchoValue::bool(false);
    };
    let Some(data) = data.string_bytes() else {
        return EchoValue::bool(false);
    };
    hash_output(algorithm, &data, raw_output.bool_value().unwrap_or(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_algos() -> EchoValue {
    let mut result = echo_value_array_new();
    for entry in HASH_ALGORITHMS {
        result =
            echo_value_array_append(result, echo_runtime_string(entry.name.as_bytes().to_vec()));
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_equals(left: EchoValue, right: EchoValue) -> EchoValue {
    match (left.string_bytes(), right.string_bytes()) {
        (Some(left), Some(right)) => EchoValue::bool(constant_time_equals(&left, &right)),
        _ => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_file(
    algorithm: EchoValue,
    filename: EchoValue,
    raw_output: EchoValue,
) -> EchoValue {
    let Some(algorithm) = algorithm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(algorithm) = parse_hash_algorithm(&algorithm) else {
        return EchoValue::bool(false);
    };
    let Some(filename) = file_bytes(filename) else {
        return EchoValue::bool(false);
    };
    hash_output(
        algorithm,
        &filename,
        raw_output.bool_value().unwrap_or(false),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_hmac(
    algorithm: EchoValue,
    data: EchoValue,
    key: EchoValue,
    raw_output: EchoValue,
) -> EchoValue {
    let Some(algorithm) = algorithm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(algorithm) = parse_hash_algorithm(&algorithm) else {
        return EchoValue::bool(false);
    };
    if !algorithm_hmac_capable(algorithm) {
        return EchoValue::bool(false);
    }
    let Some(data) = data.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(key) = key.string_bytes() else {
        return EchoValue::bool(false);
    };
    match hash_output_hmac_raw(
        algorithm,
        &key,
        &data,
        raw_output.bool_value().unwrap_or(false),
    ) {
        Some(output) => echo_runtime_string(output),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_hmac_algos() -> EchoValue {
    let mut result = echo_value_array_new();
    for entry in HASH_ALGORITHMS {
        if entry.hmac_capable {
            result = echo_value_array_append(
                result,
                echo_runtime_string(entry.name.as_bytes().to_vec()),
            );
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_hmac_file(
    algorithm: EchoValue,
    filename: EchoValue,
    key: EchoValue,
    raw_output: EchoValue,
) -> EchoValue {
    let Some(algorithm) = algorithm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(algorithm) = parse_hash_algorithm(&algorithm) else {
        return EchoValue::bool(false);
    };
    if !algorithm_hmac_capable(algorithm) {
        return EchoValue::bool(false);
    }
    let Some(filename) = file_bytes(filename) else {
        return EchoValue::bool(false);
    };
    let Some(key) = key.string_bytes() else {
        return EchoValue::bool(false);
    };
    match hash_output_hmac_raw(
        algorithm,
        &key,
        &filename,
        raw_output.bool_value().unwrap_or(false),
    ) {
        Some(output) => echo_runtime_string(output),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_hkdf(
    algorithm: EchoValue,
    ikm: EchoValue,
    length: EchoValue,
    info: EchoValue,
    salt: EchoValue,
    raw_output: EchoValue,
) -> EchoValue {
    let Some(algorithm) = algorithm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(algorithm) = parse_hash_algorithm(&algorithm) else {
        return EchoValue::bool(false);
    };
    let Some(ikm) = ikm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let info = info.string_bytes().unwrap_or_default();
    let salt = salt.string_bytes().unwrap_or_default();

    let requested_length = match length
        .php_int_value()
        .and_then(|value| usize::try_from(value).ok())
    {
        Some(value) if value == 0 => hash_algorithm_output_len(algorithm),
        Some(value) => value,
        None => hash_algorithm_output_len(algorithm),
    };

    let max_len = 255 * hash_algorithm_output_len(algorithm);
    if requested_length == 0 || requested_length > max_len {
        return EchoValue::bool(false);
    }
    if let Some(negative) = length.php_int_value() {
        if negative < 0 {
            return EchoValue::bool(false);
        }
    }

    let prk = hkdf_extract(algorithm, &salt, &ikm);
    let Some(okm) = hkdf_expand(algorithm, &prk, &info, requested_length) else {
        return EchoValue::bool(false);
    };
    echo_runtime_string(if raw_output.bool_value().unwrap_or(false) {
        okm
    } else {
        lowercase_hex_bytes(&okm)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_init(
    algorithm: EchoValue,
    options: EchoValue,
    key: EchoValue,
) -> EchoValue {
    let Some(algorithm) = algorithm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(algorithm) = parse_hash_algorithm(&algorithm) else {
        return EchoValue::bool(false);
    };
    let is_hmac = options.int_value().unwrap_or(0) == HASH_HMAC_OPTION;
    let key = if is_hmac {
        let Some(key) = key.string_bytes() else {
            return EchoValue::bool(false);
        };
        if !algorithm_hmac_capable(algorithm) {
            return EchoValue::bool(false);
        }
        key
    } else {
        Vec::new()
    };

    hash_context_new(algorithm, is_hmac, key)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_pbkdf2(
    algorithm: EchoValue,
    password: EchoValue,
    salt: EchoValue,
    iterations: EchoValue,
    length: EchoValue,
    raw_output: EchoValue,
) -> EchoValue {
    let Some(algorithm) = algorithm.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(algorithm) = parse_hash_algorithm(&algorithm) else {
        return EchoValue::bool(false);
    };
    let Some(password) = password.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(salt) = salt.string_bytes() else {
        return EchoValue::bool(false);
    };
    let iterations = match iterations.php_int_value() {
        Some(value) if value >= 1 => {
            if let Ok(iterations) = usize::try_from(value) {
                iterations
            } else {
                return EchoValue::bool(false);
            }
        }
        _ => return EchoValue::bool(false),
    };

    let requested_length = match length
        .php_int_value()
        .and_then(|value| usize::try_from(value).ok())
    {
        Some(value) if value == 0 => hash_algorithm_output_len(algorithm),
        Some(value) => value,
        None if !length.is_null() => return EchoValue::bool(false),
        None => hash_algorithm_output_len(algorithm),
    };
    if requested_length == 0 {
        return EchoValue::bool(false);
    }

    let Some(output) = pbkdf2_like(algorithm, &password, &salt, iterations, requested_length)
    else {
        return EchoValue::bool(false);
    };
    echo_runtime_string(if raw_output.bool_value().unwrap_or(false) {
        output
    } else {
        lowercase_hex_bytes(&output)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_copy(context: EchoValue) -> EchoValue {
    let Some(state) = hash_context_read(context) else {
        return EchoValue::bool(false);
    };
    hash_context_new_from_state(state)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_update(context: EchoValue, data: EchoValue) -> EchoValue {
    let Some(mut state) = hash_context_read(context) else {
        return EchoValue::bool(false);
    };
    if state.finalized {
        return EchoValue::bool(false);
    }
    let Some(data) = data.string_bytes() else {
        return EchoValue::bool(false);
    };
    state.data.extend_from_slice(&data);
    if !hash_context_write(context, &state) {
        return EchoValue::bool(false);
    }
    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_update_file(
    context: EchoValue,
    filename: EchoValue,
    _context_resource: EchoValue,
) -> EchoValue {
    let Some(mut state) = hash_context_read(context) else {
        return EchoValue::bool(false);
    };
    if state.finalized {
        return EchoValue::bool(false);
    }
    let Some(mut file_bytes) = file_bytes(filename) else {
        return EchoValue::bool(false);
    };
    state.data.append(&mut file_bytes);
    if !hash_context_write(context, &state) {
        return EchoValue::bool(false);
    }
    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_update_stream(
    context: EchoValue,
    stream: EchoValue,
    max_bytes: EchoValue,
) -> EchoValue {
    let Some(mut state) = hash_context_read(context) else {
        return EchoValue::bool(false);
    };
    if state.finalized {
        return EchoValue::bool(false);
    }

    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    if stream.file.is_none() {
        return EchoValue::bool(false);
    }

    let mut remaining = if max_bytes.is_null() {
        None
    } else {
        let Some(maximum) = max_bytes.int_value() else {
            return EchoValue::bool(false);
        };
        if maximum <= 0 {
            return EchoValue::bool(false);
        }
        let Ok(maximum) = usize::try_from(maximum) else {
            return EchoValue::bool(false);
        };
        Some(maximum)
    };

    let chunk_size = 4096_usize;
    loop {
        let to_read = match remaining {
            Some(remaining) => remaining.min(chunk_size),
            None => chunk_size,
        };
        if to_read == 0 {
            break;
        }

        let Some(file) = stream.file.as_mut() else {
            return EchoValue::bool(false);
        };
        let mut chunk = vec![0_u8; to_read];
        let read = match file.read(&mut chunk) {
            Ok(read) => read,
            Err(_) => return EchoValue::bool(false),
        };
        if read == 0 {
            break;
        }
        chunk.truncate(read);
        state.data.extend_from_slice(&chunk);
        if let Some(remaining) = remaining.as_mut() {
            *remaining = remaining.saturating_sub(read);
            if *remaining == 0 {
                break;
            }
        }
        if read < to_read {
            break;
        }
    }

    if !hash_context_write(context, &state) {
        return EchoValue::bool(false);
    }

    EchoValue::bool(!state.finalized)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hash_final(context: EchoValue, raw_output: EchoValue) -> EchoValue {
    let mut state = match hash_context_read(context) {
        Some(state) => state,
        None => return EchoValue::bool(false),
    };
    if state.finalized {
        return EchoValue::bool(false);
    }

    let Some(algorithm) = parse_hash_algorithm(&state.algorithm) else {
        return EchoValue::bool(false);
    };
    let data = if state.is_hmac {
        let Some(output) = hash_output_hmac_raw(algorithm, &state.key, &state.data, true) else {
            return EchoValue::bool(false);
        };
        output
    } else {
        let Some(output) = hash_output_raw(algorithm, &state.data) else {
            return EchoValue::bool(false);
        };
        output
    };

    state.finalized = true;
    if !hash_context_write(context, &state) {
        return EchoValue::bool(false);
    }
    echo_runtime_string(if raw_output.bool_value().unwrap_or(false) {
        data
    } else {
        lowercase_hex_bytes(&data)
    })
}

fn hash_output_raw(algorithm: HashAlgorithm, data: &[u8]) -> Option<Vec<u8>> {
    match algorithm {
        HashAlgorithm::Md5 => Some(<Md5 as Md5Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha1 => Some(<Sha1 as Sha1Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha224 => Some(<Sha224 as Sha2Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha256 => Some(<Sha256 as Sha2Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha384 => Some(<Sha384 as Sha2Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha512 => Some(<Sha512 as Sha2Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha512_224 => Some(<Sha512_224 as Sha2Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha512_256 => Some(<Sha512_256 as Sha2Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha3_224 => Some(<Sha3_224 as Sha3Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha3_256 => Some(<Sha3_256 as Sha3Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha3_384 => Some(<Sha3_384 as Sha3Digest>::digest(data).to_vec()),
        HashAlgorithm::Sha3_512 => Some(<Sha3_512 as Sha3Digest>::digest(data).to_vec()),
        HashAlgorithm::Crc32 => Some(crc32_raw_output(crc32_fast(data))),
        HashAlgorithm::Crc32b => Some(crc32_raw_output(crc32b_fast(data))),
        HashAlgorithm::Adler32 => Some(adler32_raw_output(adler32_fast(data))),
    }
}

fn hash_output_hmac_raw(
    algorithm: HashAlgorithm,
    key: &[u8],
    data: &[u8],
    raw_output: bool,
) -> Option<Vec<u8>> {
    let output = match algorithm {
        HashAlgorithm::Md5 => hmac_bytes(key, data, 64, HashAlgorithm::Md5),
        HashAlgorithm::Sha1 => hmac_bytes(key, data, 64, HashAlgorithm::Sha1),
        HashAlgorithm::Sha224 => hmac_bytes(key, data, 64, HashAlgorithm::Sha224),
        HashAlgorithm::Sha256 => hmac_bytes(key, data, 64, HashAlgorithm::Sha256),
        HashAlgorithm::Sha384 => hmac_bytes(key, data, 128, HashAlgorithm::Sha384),
        HashAlgorithm::Sha512 => hmac_bytes(key, data, 128, HashAlgorithm::Sha512),
        HashAlgorithm::Sha512_224 => hmac_bytes(key, data, 128, HashAlgorithm::Sha512_224),
        HashAlgorithm::Sha512_256 => hmac_bytes(key, data, 128, HashAlgorithm::Sha512_256),
        HashAlgorithm::Sha3_224 => hmac_bytes(key, data, 144, HashAlgorithm::Sha3_224),
        HashAlgorithm::Sha3_256 => hmac_bytes(key, data, 136, HashAlgorithm::Sha3_256),
        HashAlgorithm::Sha3_384 => hmac_bytes(key, data, 104, HashAlgorithm::Sha3_384),
        HashAlgorithm::Sha3_512 => hmac_bytes(key, data, 72, HashAlgorithm::Sha3_512),
        HashAlgorithm::Crc32 | HashAlgorithm::Crc32b | HashAlgorithm::Adler32 => None,
    }?;

    if raw_output {
        Some(output)
    } else {
        Some(lowercase_hex_bytes(&output))
    }
}

fn hmac_bytes(
    key: &[u8],
    data: &[u8],
    block_size: usize,
    algorithm: HashAlgorithm,
) -> Option<Vec<u8>> {
    let key = if key.len() > block_size {
        hash_output_raw(algorithm, key)?
    } else {
        key.to_vec()
    };
    let mut ipad = vec![0x36_u8; block_size];
    let mut opad = vec![0x5c_u8; block_size];
    for index in 0..key.len().min(block_size) {
        ipad[index] ^= key[index];
        opad[index] ^= key[index];
    }

    let mut inner = ipad;
    inner.extend_from_slice(data);
    let inner = hash_output_raw(algorithm, &inner)?;
    let mut outer = opad;
    outer.extend_from_slice(&inner);
    hash_output_raw(algorithm, &outer)
}

fn hash_output(algorithm: HashAlgorithm, data: &[u8], raw_output: bool) -> EchoValue {
    match hash_output_raw(algorithm, data) {
        Some(raw) => echo_runtime_string(if raw_output {
            raw
        } else {
            lowercase_hex_bytes(&raw)
        }),
        None => EchoValue::bool(false),
    }
}

fn parse_hash_algorithm(bytes: &[u8]) -> Option<HashAlgorithm> {
    let mut normalized = bytes.to_vec();
    for byte in &mut normalized {
        byte.make_ascii_lowercase();
    }
    match normalized.as_slice() {
        b"md5" => Some(HashAlgorithm::Md5),
        b"sha1" => Some(HashAlgorithm::Sha1),
        b"sha224" => Some(HashAlgorithm::Sha224),
        b"sha256" => Some(HashAlgorithm::Sha256),
        b"sha384" => Some(HashAlgorithm::Sha384),
        b"sha512" => Some(HashAlgorithm::Sha512),
        b"sha512/224" => Some(HashAlgorithm::Sha512_224),
        b"sha512/256" => Some(HashAlgorithm::Sha512_256),
        b"sha3-224" => Some(HashAlgorithm::Sha3_224),
        b"sha3-256" => Some(HashAlgorithm::Sha3_256),
        b"sha3-384" => Some(HashAlgorithm::Sha3_384),
        b"sha3-512" => Some(HashAlgorithm::Sha3_512),
        b"crc32" => Some(HashAlgorithm::Crc32),
        b"crc32b" => Some(HashAlgorithm::Crc32b),
        b"adler32" => Some(HashAlgorithm::Adler32),
        _ => None,
    }
}

fn algorithm_hmac_capable(algorithm: HashAlgorithm) -> bool {
    HASH_ALGORITHMS
        .iter()
        .find(|entry| entry.algorithm == algorithm)
        .is_some_and(|entry| entry.hmac_capable)
}

fn hash_value_algorithm(name: &str) -> HashAlgorithm {
    parse_hash_algorithm(name.as_bytes()).unwrap_or(HashAlgorithm::Md5)
}

fn hash_algorithm_output_len(algorithm: HashAlgorithm) -> usize {
    match algorithm {
        HashAlgorithm::Md5 => 16,
        HashAlgorithm::Sha1 => 20,
        HashAlgorithm::Sha224 => 28,
        HashAlgorithm::Sha256 => 32,
        HashAlgorithm::Sha384 => 48,
        HashAlgorithm::Sha512 => 64,
        HashAlgorithm::Sha512_224 => 28,
        HashAlgorithm::Sha512_256 => 32,
        HashAlgorithm::Sha3_224 => 28,
        HashAlgorithm::Sha3_256 => 32,
        HashAlgorithm::Sha3_384 => 48,
        HashAlgorithm::Sha3_512 => 64,
        HashAlgorithm::Crc32 | HashAlgorithm::Crc32b => 4,
        HashAlgorithm::Adler32 => 4,
    }
}

fn crc32_fast(bytes: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

fn crc32b_fast(bytes: &[u8]) -> u32 {
    crc32_fast(bytes)
}

fn adler32_fast(bytes: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for byte in bytes {
        a = (a + (*byte as u32)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn crc32_raw_output(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn adler32_raw_output(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

fn hash_context_new(algorithm: HashAlgorithm, is_hmac: bool, key: Vec<u8>) -> EchoValue {
    let state = HashContext {
        algorithm: algorithm_name_bytes(algorithm),
        is_hmac,
        key,
        data: Vec::new(),
        finalized: false,
    };
    hash_context_new_from_state(state)
}

fn hash_context_new_from_state(state: HashContext) -> EchoValue {
    let context = echo_value_object_new();
    let Some(context) = (unsafe { (context.payload as *mut EchoObject).as_mut() }) else {
        return EchoValue::bool(false);
    };
    hash_context_set_field(
        context,
        HASH_CTX_ALGORITHM_FIELD,
        echo_runtime_string(state.algorithm),
    );
    hash_context_set_field(
        context,
        HASH_CTX_DATA_FIELD,
        echo_runtime_string(state.data),
    );
    hash_context_set_field(
        context,
        HASH_CTX_FINALIZED_FIELD,
        EchoValue::bool(state.finalized),
    );
    hash_context_set_field(context, HASH_CTX_HMAC_FIELD, EchoValue::bool(state.is_hmac));
    if state.is_hmac {
        hash_context_set_field(context, HASH_CTX_KEY_FIELD, echo_runtime_string(state.key));
    } else {
        hash_context_set_field(context, HASH_CTX_KEY_FIELD, EchoValue::null());
    }
    EchoValue::object(context)
}

fn hash_context_set_field(context: &mut EchoObject, field: &str, value: EchoValue) {
    for (name, slot) in context.fields.iter_mut().rev() {
        if name == field {
            *slot = value;
            return;
        }
    }
    context.fields.push((field.to_string(), value));
}

fn hash_context_get_field(context: &EchoObject, field: &str) -> Option<EchoValue> {
    context
        .fields
        .iter()
        .rev()
        .find_map(|(name, value)| (name == field).then_some(*value))
}

fn hash_context_read(context: EchoValue) -> Option<HashContext> {
    if !context.is_object() {
        return None;
    }
    let object = unsafe { (context.payload as *const EchoObject).as_ref() }?;
    let algorithm = {
        let algorithm = hash_context_get_field(object, HASH_CTX_ALGORITHM_FIELD)?;
        algorithm.string_bytes()?
    };
    let is_hmac = hash_context_get_field(object, HASH_CTX_HMAC_FIELD)
        .and_then(|value| value.bool_value())
        .unwrap_or(false);
    let finalized = hash_context_get_field(object, HASH_CTX_FINALIZED_FIELD)
        .and_then(|value| value.bool_value())
        .unwrap_or(false);
    let key = if is_hmac {
        hash_context_get_field(object, HASH_CTX_KEY_FIELD)
            .and_then(|value| value.string_bytes())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let data = hash_context_get_field(object, HASH_CTX_DATA_FIELD)
        .and_then(|value| value.string_bytes())
        .unwrap_or_default();
    Some(HashContext {
        algorithm,
        is_hmac,
        key,
        data,
        finalized,
    })
}

fn hash_context_write(context: EchoValue, state: &HashContext) -> bool {
    let Some(context) = (unsafe { (context.payload as *mut EchoObject).as_mut() }) else {
        return false;
    };
    hash_context_set_field(
        context,
        HASH_CTX_ALGORITHM_FIELD,
        echo_runtime_string(state.algorithm.clone()),
    );
    hash_context_set_field(
        context,
        HASH_CTX_DATA_FIELD,
        echo_runtime_string(state.data.clone()),
    );
    hash_context_set_field(
        context,
        HASH_CTX_FINALIZED_FIELD,
        EchoValue::bool(state.finalized),
    );
    hash_context_set_field(context, HASH_CTX_HMAC_FIELD, EchoValue::bool(state.is_hmac));
    if state.is_hmac {
        hash_context_set_field(
            context,
            HASH_CTX_KEY_FIELD,
            echo_runtime_string(state.key.clone()),
        );
    } else {
        hash_context_set_field(context, HASH_CTX_KEY_FIELD, EchoValue::null());
    }
    true
}

fn algorithm_name_bytes(algorithm: HashAlgorithm) -> Vec<u8> {
    HASH_ALGORITHMS
        .iter()
        .find(|entry| entry.algorithm == algorithm)
        .map(|entry| entry.name.as_bytes().to_vec())
        .unwrap_or_else(|| b"md5".to_vec())
}

fn file_bytes(filename: EchoValue) -> Option<Vec<u8>> {
    let Some(path) = filename.string_bytes() else {
        return None;
    };
    let path = String::from_utf8_lossy(&path);
    fs::read(path.as_ref()).ok()
}

fn pbkdf2_like(
    algorithm: HashAlgorithm,
    password: &[u8],
    salt: &[u8],
    iterations: usize,
    output_length: usize,
) -> Option<Vec<u8>> {
    if iterations == 0 || output_length == 0 {
        return None;
    }
    let hash_len = hash_algorithm_output_len(algorithm);
    let max_blocks = 255_usize;
    let block_count = output_length.div_ceil(hash_len);
    if block_count == 0 || block_count > max_blocks {
        return None;
    }
    let mut output = Vec::with_capacity(output_length);

    for block in 1..=block_count {
        let mut input = Vec::from(salt);
        input.extend_from_slice(&u32::try_from(block).ok()?.to_be_bytes());
        let mut u = hash_output_hmac_raw(algorithm, password, &input, true)?;
        let mut acc = u.clone();
        for _ in 1..iterations {
            u = hash_output_hmac_raw(algorithm, password, &u, true)?;
            for (a, b) in acc.iter_mut().zip(&u) {
                *a ^= *b;
            }
        }
        output.extend_from_slice(&acc);
    }
    output.truncate(output_length);
    Some(output)
}

fn hkdf_extract(algorithm: HashAlgorithm, salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    if salt.is_empty() {
        let zeroes = vec![0_u8; hash_algorithm_output_len(algorithm)];
        hash_output_hmac_raw(algorithm, &zeroes, ikm, true).unwrap_or_default()
    } else {
        hash_output_hmac_raw(algorithm, salt, ikm, true).unwrap_or_default()
    }
}

fn hkdf_expand(
    algorithm: HashAlgorithm,
    prk: &[u8],
    info: &[u8],
    length: usize,
) -> Option<Vec<u8>> {
    let hash_len = hash_algorithm_output_len(algorithm);
    let block_count = length.div_ceil(hash_len);
    if block_count == 0 || block_count > 255 {
        return None;
    }
    let mut output = Vec::with_capacity(length);
    let mut previous = Vec::new();

    for block in 1..=block_count {
        let mut input = previous;
        input.extend_from_slice(info);
        input.push(u8::try_from(block).ok()?);
        previous = hash_output_hmac_raw(algorithm, prk, &input, true)?;
        output.extend_from_slice(&previous);
    }

    output.truncate(length);
    Some(output)
}
