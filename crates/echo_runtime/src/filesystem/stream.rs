use crate::{EchoValue, echo_runtime_string, filesystem::path_buf_from_bytes};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct EchoFileStream {
    pub file: Option<File>,
    delete_on_close: Option<PathBuf>,
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fopen(
    filename: EchoValue,
    mode: EchoValue,
    _use_include_path: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(filename) = filename.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(mode) = mode.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(options) = fopen_options_from_mode(&mode) else {
        return EchoValue::bool(false);
    };
    let Some(path) = fopen_path(&filename) else {
        return EchoValue::bool(false);
    };

    match options.open(path) {
        Ok(file) => {
            let stream = Box::into_raw(Box::new(EchoFileStream {
                file: Some(file),
                delete_on_close: None,
            }));
            EchoValue::file_stream(stream)
        }
        Err(_) => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fread(stream: EchoValue, length: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::bool(false);
    };
    if length <= 0 {
        return EchoValue::bool(false);
    }
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::bool(false);
    };

    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };
    let mut bytes = vec![0_u8; length];
    let read = match file.read(&mut bytes) {
        Ok(size) => size,
        Err(_) => return EchoValue::bool(false),
    };
    bytes.truncate(read);
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fclose(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    match stream.file.take() {
        Some(_) => {
            if let Some(path) = stream.delete_on_close.take() {
                let _ = std::fs::remove_file(path);
            }
            EchoValue::bool(true)
        }
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_tmpfile() -> EchoValue {
    let temp_dir = std::env::temp_dir();
    let pid = std::process::id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_nanos())
        .unwrap_or_default();

    for index in 0_u64..1024 {
        let name = format!("echo-runtime-tmpfile-{pid}-{now}-{index}");
        let mut path = temp_dir.clone();
        path.push(name);
        let result = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path);
        if let Ok(file) = result {
            let stream = Box::into_raw(Box::new(EchoFileStream {
                file: Some(file),
                delete_on_close: Some(path),
            }));
            return EchoValue::file_stream(stream);
        }
    }

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stream_get_contents(
    stream: EchoValue,
    length: EchoValue,
    offset: EchoValue,
) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    if let Some(offset) = offset.php_int_value()
        && offset >= 0
    {
        let Ok(offset) = u64::try_from(offset) else {
            return EchoValue::bool(false);
        };
        if file.seek(SeekFrom::Start(offset)).is_err() {
            return EchoValue::bool(false);
        }
    }

    let mut bytes = Vec::new();
    if length.is_null() {
        if file.read_to_end(&mut bytes).is_err() {
            return EchoValue::bool(false);
        }
    } else {
        let Some(length) = length.php_int_value() else {
            return EchoValue::bool(false);
        };
        if length < 0 {
            return EchoValue::bool(false);
        }
        let Ok(length) = usize::try_from(length) else {
            return EchoValue::bool(false);
        };
        let mut limited = file.take(length as u64);
        if limited.read_to_end(&mut bytes).is_err() {
            return EchoValue::bool(false);
        }
    }
    echo_runtime_string(bytes)
}

fn fopen_path(filename: &[u8]) -> Option<PathBuf> {
    path_buf_from_bytes(filename)
}

fn fopen_options_from_mode(mode: &[u8]) -> Option<OpenOptions> {
    if mode.is_empty() {
        return None;
    }

    let mode = std::str::from_utf8(mode).ok()?.trim();
    let mut normalized = String::with_capacity(mode.len());
    for value in mode.bytes() {
        if matches!(value, b'b' | b'B' | b't' | b'T') {
            continue;
        }
        normalized.push(value as char);
    }
    if normalized.is_empty() || normalized.len() > 2 {
        return None;
    }

    let mut chars = normalized.chars();
    let base = chars.next()?;
    if !matches!(base, 'r' | 'w' | 'a') {
        return None;
    }
    if !chars.all(|value| value == '+') {
        return None;
    }
    let has_plus = normalized.contains('+');

    let mut options = OpenOptions::new();
    match base {
        'r' => {
            options.read(true);
            options.write(has_plus);
        }
        'w' => {
            options.write(true);
            options.create(true);
            options.truncate(true);
            options.read(has_plus);
        }
        'a' => {
            options.write(true);
            options.create(true);
            options.append(true);
            options.read(has_plus);
        }
        _ => return None,
    }

    Some(options)
}
