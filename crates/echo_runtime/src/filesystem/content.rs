use std::fs::OpenOptions;
use std::io::Write;

use crate::{EchoValue, echo_runtime_string, write_runtime_output};

use super::path_buf_from_bytes;

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_file_get_contents(
    filename: EchoValue,
    _use_include_path: EchoValue,
    _context: EchoValue,
    offset: EchoValue,
    length: EchoValue,
) -> EchoValue {
    let Some(filename) = filename.string_bytes() else {
        return EchoValue::error();
    };
    let offset = offset.php_int_value().unwrap_or(0);
    let length = if length.is_null() {
        None
    } else {
        match length.php_int_value() {
            Some(value) if value >= 0 => Some(value as usize),
            Some(_) => return EchoValue::bool(false),
            None => None,
        }
    };

    path_file_get_contents(&filename, offset, length)
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_file_put_contents(
    filename: EchoValue,
    data: EchoValue,
    flags: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(filename) = filename.string_bytes() else {
        return EchoValue::error();
    };
    let Some(data) = data.string_bytes() else {
        return EchoValue::error();
    };
    let flags = flags.php_int_value().unwrap_or(0);

    path_file_put_contents(&filename, &data, flags)
        .and_then(|written| i64::try_from(written).ok())
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_readfile(
    filename: EchoValue,
    _use_include_path: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(filename) = filename.string_bytes() else {
        return EchoValue::error();
    };

    let Some(bytes) = path_file_get_contents(&filename, 0, None) else {
        return EchoValue::bool(false);
    };
    write_runtime_output(&bytes);

    i64::try_from(bytes.len())
        .map(EchoValue::int)
        .unwrap_or_else(|_| EchoValue::bool(false))
}

fn path_file_get_contents(bytes: &[u8], offset: i64, length: Option<usize>) -> Option<Vec<u8>> {
    let path = path_buf_from_bytes(bytes)?;
    let data = std::fs::read(path).ok()?;
    let start = if offset >= 0 {
        usize::try_from(offset).ok()?
    } else {
        let from_end = usize::try_from(offset.unsigned_abs()).ok()?;
        data.len().checked_sub(from_end)?
    };
    if start > data.len() {
        return None;
    }

    let end = length
        .and_then(|length| start.checked_add(length))
        .map(|end| end.min(data.len()))
        .unwrap_or(data.len());
    Some(data[start..end].to_vec())
}

pub(crate) const PHP_FILE_APPEND: i64 = 8;

fn path_file_put_contents(bytes: &[u8], data: &[u8], flags: i64) -> Option<usize> {
    let path = path_buf_from_bytes(bytes)?;
    let append = flags & PHP_FILE_APPEND != 0;
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options.open(path).ok()?;
    file.write_all(data).ok()?;
    Some(data.len())
}
