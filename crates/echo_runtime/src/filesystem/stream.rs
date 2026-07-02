use crate::{
    EchoArray, EchoValue, echo_runtime_string,
    filesystem::{path_buf_from_bytes, php_stat_from_metadata, stat_array},
    write_runtime_output,
};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct EchoFileStream {
    pub file: Option<File>,
    eof: bool,
    delete_on_close: Option<PathBuf>,
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stream_get_wrappers() -> EchoValue {
    string_array(&[b"php", b"file"])
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stream_get_transports() -> EchoValue {
    string_array(&[b"tcp", b"udp", b"unix", b"udg"])
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stream_get_filters() -> EchoValue {
    string_array(&[b"string.rot13", b"string.toupper", b"string.tolower"])
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
                eof: false,
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
        Ok(size) => {
            stream.eof = size == 0;
            size
        }
        Err(_) => return EchoValue::bool(false),
    };
    bytes.truncate(read);
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fgetc(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    let mut byte = [0_u8; 1];
    match file.read(&mut byte) {
        Ok(1) => {
            stream.eof = false;
            echo_runtime_string(byte.to_vec())
        }
        Ok(_) => {
            stream.eof = true;
            EchoValue::bool(false)
        }
        Err(_) => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fgets(stream: EchoValue, length: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    let max_bytes = if length.is_null() {
        None
    } else {
        let Some(length) = length.php_int_value() else {
            return EchoValue::bool(false);
        };
        if length <= 1 {
            return EchoValue::bool(false);
        }
        let Ok(length) = usize::try_from(length) else {
            return EchoValue::bool(false);
        };

        Some(length - 1)
    };

    let mut bytes = Vec::new();
    while max_bytes.is_none_or(|max_bytes| bytes.len() < max_bytes) {
        let mut byte = [0_u8; 1];
        match file.read(&mut byte) {
            Ok(1) => {
                stream.eof = false;
                bytes.push(byte[0]);
                if byte[0] == b'\n' {
                    break;
                }
            }
            Ok(_) => {
                stream.eof = true;
                break;
            }
            Err(_) => return EchoValue::bool(false),
        }
    }

    if bytes.is_empty() {
        return EchoValue::bool(false);
    }

    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_feof(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    if stream.file.is_none() {
        return EchoValue::bool(false);
    }

    EchoValue::bool(stream.eof)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fflush(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(file.flush().is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fsync(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(file.flush().and_then(|()| file.sync_all()).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ftruncate(stream: EchoValue, size: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };
    let Some(size) = size.php_int_value() else {
        return EchoValue::bool(false);
    };
    let Ok(size) = u64::try_from(size) else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(file.set_len(size).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fdatasync(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(file.flush().and_then(|()| file.sync_data()).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fwrite(
    stream: EchoValue,
    data: EchoValue,
    length: EchoValue,
) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };
    let Some(data) = data.string_bytes() else {
        return EchoValue::bool(false);
    };

    let bytes = if length.is_null() {
        data
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
        data[..data.len().min(length)].to_vec()
    };

    match file.write(&bytes) {
        Ok(written) => i64::try_from(written)
            .map(EchoValue::int)
            .unwrap_or_else(|_| EchoValue::bool(false)),
        Err(_) => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fpassthru(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    let mut bytes = Vec::new();
    match file.read_to_end(&mut bytes) {
        Ok(_) => {
            stream.eof = true;
            write_runtime_output(&bytes);
            i64::try_from(bytes.len())
                .map(EchoValue::int)
                .unwrap_or_else(|_| EchoValue::bool(false))
        }
        Err(_) => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fstat(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_ref() else {
        return EchoValue::bool(false);
    };

    file.metadata()
        .ok()
        .and_then(php_stat_from_metadata)
        .map(stat_array)
        .unwrap_or_else(|| EchoValue::bool(false))
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
pub extern "C" fn echo_php_ftell(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    match file.stream_position() {
        Ok(position) => i64::try_from(position)
            .map(EchoValue::int)
            .unwrap_or_else(|_| EchoValue::bool(false)),
        Err(_) => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fseek(stream: EchoValue, offset: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::int(-1);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::int(-1);
    };
    let Some(offset) = offset.php_int_value() else {
        return EchoValue::int(-1);
    };
    let Ok(offset) = u64::try_from(offset) else {
        return EchoValue::int(-1);
    };

    match file.seek(SeekFrom::Start(offset)) {
        Ok(_) => {
            stream.eof = false;
            EchoValue::int(0)
        }
        Err(_) => EchoValue::int(-1),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rewind(stream: EchoValue) -> EchoValue {
    let Some(stream) = stream.as_stream_mut() else {
        return EchoValue::bool(false);
    };
    let Some(file) = stream.file.as_mut() else {
        return EchoValue::bool(false);
    };

    match file.seek(SeekFrom::Start(0)) {
        Ok(_) => {
            stream.eof = false;
            EchoValue::bool(true)
        }
        Err(_) => EchoValue::bool(false),
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
                eof: false,
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
        match file.read_to_end(&mut bytes) {
            Ok(_) => stream.eof = true,
            Err(_) => return EchoValue::bool(false),
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
        match limited.read_to_end(&mut bytes) {
            Ok(read) => stream.eof = read < length,
            Err(_) => return EchoValue::bool(false),
        }
    }
    echo_runtime_string(bytes)
}

fn fopen_path(filename: &[u8]) -> Option<PathBuf> {
    path_buf_from_bytes(filename)
}

fn string_array(values: &[&[u8]]) -> EchoValue {
    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        values
            .iter()
            .map(|value| echo_runtime_string(value.to_vec()))
            .collect(),
    ))))
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
