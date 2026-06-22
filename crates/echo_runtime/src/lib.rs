pub mod abi;
mod assertions;
mod callable;
mod collections;
mod encoding;
mod environment;
pub mod error;
mod execution;
mod filesystem;
pub mod io;
mod math;
pub mod net;
mod output;
pub mod poll;
pub mod process;
mod reflection;
mod require;
pub mod sched;
mod string;
pub mod task;
pub mod task_group;
pub mod thread;
pub mod time;
pub mod value;

#[cfg(test)]
use std::env;
#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

pub use assertions::{echo_std_assert_equals, echo_std_assert_ok};
pub use callable::{
    EchoCallable, EchoSymbol, echo_call, echo_call_function, echo_normalize_callable,
    echo_php_define,
};
pub use collections::{
    EchoArray, EchoList, echo_php_array_chunk, echo_php_array_combine, echo_php_array_count_values,
    echo_php_array_fill, echo_php_array_fill_keys, echo_php_array_flip, echo_php_array_is_list,
    echo_php_array_key_exists, echo_php_array_key_first, echo_php_array_key_last,
    echo_php_array_keys, echo_php_array_merge, echo_php_array_pad, echo_php_array_product,
    echo_php_array_replace, echo_php_array_reverse, echo_php_array_search, echo_php_array_slice,
    echo_php_array_sum, echo_php_array_values, echo_php_count, echo_php_in_array,
    echo_value_array_append, echo_value_array_new, echo_value_array_set, echo_value_index_get,
    echo_value_list_append, echo_value_list_new,
};
use collections::{EchoArrayKey, echo_arrays_equal, echo_lists_equal, php_array_union};
pub use encoding::{
    echo_php_base64_decode, echo_php_base64_encode, echo_php_bin2hex, echo_php_crc32,
    echo_php_escapeshellarg, echo_php_escapeshellcmd, echo_php_hex2bin, echo_php_md5,
    echo_php_rawurldecode, echo_php_rawurlencode, echo_php_sha1, echo_php_urldecode,
    echo_php_urlencode,
};
pub use environment::*;
pub use error::EchoError;
pub use execution::echo_join;
#[cfg(test)]
use filesystem::{
    PHP_FILE_APPEND, path_buf_from_bytes, path_getcwd, path_is_dir, path_is_file, path_realpath,
};
pub use filesystem::{
    echo_php_basename, echo_php_chdir, echo_php_copy, echo_php_dirname, echo_php_file_exists,
    echo_php_file_get_contents, echo_php_file_put_contents, echo_php_fileatime, echo_php_filectime,
    echo_php_filegroup, echo_php_fileinode, echo_php_filemtime, echo_php_fileowner,
    echo_php_fileperms, echo_php_filesize, echo_php_filetype, echo_php_getcwd, echo_php_is_dir,
    echo_php_is_executable, echo_php_is_file, echo_php_is_link, echo_php_is_readable,
    echo_php_is_writable, echo_php_link, echo_php_mkdir, echo_php_readfile, echo_php_readlink,
    echo_php_realpath, echo_php_rename, echo_php_rmdir, echo_php_symlink,
    echo_php_sys_get_temp_dir, echo_php_tempnam, echo_php_touch, echo_php_uniqid, echo_php_unlink,
};
use math::echo_math_pow;
pub use math::{
    echo_php_acos, echo_php_acosh, echo_php_asin, echo_php_asinh, echo_php_atan, echo_php_atan2,
    echo_php_atanh, echo_php_ceil, echo_php_cos, echo_php_cosh, echo_php_deg2rad, echo_php_exp,
    echo_php_expm1, echo_php_fdiv, echo_php_floor, echo_php_fmod, echo_php_fpow, echo_php_hypot,
    echo_php_is_finite, echo_php_is_infinite, echo_php_is_nan, echo_php_log, echo_php_log1p,
    echo_php_log10, echo_php_pi, echo_php_pow, echo_php_rad2deg, echo_php_sin, echo_php_sinh,
    echo_php_sqrt, echo_php_tan, echo_php_tanh,
};
pub use net::{
    echo_std_http_read_request, echo_std_http_response_text, echo_std_net_accept,
    echo_std_net_close, echo_std_net_connect, echo_std_net_listen, echo_std_net_read,
    echo_std_net_write,
};
use output::reset_output_runtime;
pub(crate) use output::write_runtime_output;
pub use output::{
    OutputRuntime, echo_php_flush, echo_php_ob_clean, echo_php_ob_end_clean, echo_php_ob_end_flush,
    echo_php_ob_flush, echo_php_ob_get_clean, echo_php_ob_get_contents, echo_php_ob_get_flush,
    echo_php_ob_get_length, echo_php_ob_get_level, echo_php_ob_implicit_flush, echo_php_ob_start,
    echo_php_ob_start_value, echo_shutdown, echo_write, echo_write_i64, echo_write_i64_or_false,
    echo_write_string, echo_write_value,
};
pub use process::{echo_process_join, echo_process_spawn};
#[cfg(test)]
use reflection::REFLECTION_SOURCE_PHP_BUILTIN;
pub use reflection::{
    echo_php_function_exists, echo_php_is_callable, echo_reflection_register_function,
    echo_std_reflect_exists, echo_std_reflect_params, echo_std_reflect_return_type,
    echo_std_reflect_type_of,
};
pub use require::{echo_php_require, echo_php_require_once};
pub use string::{
    echo_php_addslashes, echo_php_chr, echo_php_chunk_split, echo_php_decbin, echo_php_dechex,
    echo_php_decoct, echo_php_explode, echo_php_implode, echo_php_lcfirst, echo_php_ltrim,
    echo_php_nl2br, echo_php_ord, echo_php_quoted_printable_decode,
    echo_php_quoted_printable_encode, echo_php_quotemeta, echo_php_rtrim, echo_php_str_contains,
    echo_php_str_ends_with, echo_php_str_ireplace, echo_php_str_pad, echo_php_str_repeat,
    echo_php_str_replace, echo_php_str_rot13, echo_php_str_split, echo_php_str_starts_with,
    echo_php_strcasecmp, echo_php_strcmp, echo_php_strcspn, echo_php_stripos,
    echo_php_stripslashes, echo_php_stristr, echo_php_strlen, echo_php_strncasecmp,
    echo_php_strncmp, echo_php_strpbrk, echo_php_strpos, echo_php_strrchr, echo_php_strrev,
    echo_php_strripos, echo_php_strrpos, echo_php_strspn, echo_php_strstr, echo_php_strtolower,
    echo_php_strtoupper, echo_php_strtr, echo_php_strval, echo_php_substr, echo_php_substr_compare,
    echo_php_substr_count, echo_php_trim, echo_php_ucfirst, echo_php_ucwords, echo_value_concat,
    echo_value_string,
};
use string::{php_string_to_number_builtin, trim_ascii, trim_ascii_start};
pub use task::{echo_task_defer, echo_task_join, echo_task_run, echo_task_sleep_current};
pub use task_group::{echo_task_group_add, echo_task_group_new, echo_task_group_run_and_join};
pub use thread::{echo_thread_fork, echo_thread_fork_task, echo_thread_join};
pub use time::{echo_php_microtime, echo_time_sleep};
use value::format_php_float;
pub use value::{
    EchoObject, EchoString, echo_php_boolval, echo_php_floatval, echo_php_gettype, echo_php_intval,
    echo_php_is_array, echo_php_is_bool, echo_php_is_countable, echo_php_is_float, echo_php_is_int,
    echo_php_is_iterable, echo_php_is_null, echo_php_is_object, echo_php_is_resource,
    echo_php_is_scalar, echo_php_is_string, echo_value_object_get, echo_value_object_new,
    echo_value_object_set,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EchoValue {
    pub kind: i32,
    pub payload: u64,
}

const ECHO_VALUE_NULL: i32 = 0;
const ECHO_VALUE_ERROR: i32 = -1;
const ECHO_VALUE_BOOL: i32 = 1;
const ECHO_VALUE_INT: i32 = 2;
const ECHO_VALUE_STRING: i32 = 3;
const ECHO_VALUE_ARRAY: i32 = 4;
const ECHO_VALUE_TASK: i32 = 5;
const ECHO_VALUE_PENDING: i32 = 6;
const ECHO_VALUE_TCP_LISTENER: i32 = 7;
const ECHO_VALUE_TCP_CONNECTION: i32 = 8;
const ECHO_VALUE_OBJECT: i32 = 9;
const ECHO_VALUE_LIST: i32 = 10;
const ECHO_VALUE_FLOAT: i32 = 11;
const ECHO_VALUE_PROCESS: i32 = 12;
const ECHO_VALUE_THREAD: i32 = 13;
const ECHO_VALUE_TASK_GROUP: i32 = 14;

const INSPECT_MAX_DEPTH: usize = 3;
const INSPECT_MAX_ITEMS: usize = 8;

impl EchoValue {
    pub const fn null() -> Self {
        Self {
            kind: ECHO_VALUE_NULL,
            payload: 0,
        }
    }

    pub const fn error() -> Self {
        Self {
            kind: ECHO_VALUE_ERROR,
            payload: 0,
        }
    }

    pub const fn bool(value: bool) -> Self {
        Self {
            kind: ECHO_VALUE_BOOL,
            payload: value as u64,
        }
    }

    pub const fn int(value: i64) -> Self {
        Self {
            kind: ECHO_VALUE_INT,
            payload: value as u64,
        }
    }

    pub const fn float(value: f64) -> Self {
        Self {
            kind: ECHO_VALUE_FLOAT,
            payload: value.to_bits(),
        }
    }

    pub const fn is_null(self) -> bool {
        self.kind == ECHO_VALUE_NULL
    }

    pub const fn is_false(self) -> bool {
        self.kind == ECHO_VALUE_BOOL && self.payload == 0
    }

    pub const fn is_bool(self) -> bool {
        self.kind == ECHO_VALUE_BOOL
    }

    pub const fn is_int(self) -> bool {
        self.kind == ECHO_VALUE_INT
    }

    pub const fn is_float(self) -> bool {
        self.kind == ECHO_VALUE_FLOAT
    }

    pub fn string(value: *mut EchoString) -> Self {
        Self {
            kind: ECHO_VALUE_STRING,
            payload: value as u64,
        }
    }

    pub fn task(value: *mut task::EchoTask) -> Self {
        Self {
            kind: ECHO_VALUE_TASK,
            payload: value as u64,
        }
    }

    pub fn task_group(value: *mut task_group::EchoTaskGroup) -> Self {
        Self {
            kind: ECHO_VALUE_TASK_GROUP,
            payload: value as u64,
        }
    }

    pub fn process(value: *mut process::EchoProcess) -> Self {
        Self {
            kind: ECHO_VALUE_PROCESS,
            payload: value as u64,
        }
    }

    pub fn thread(value: *mut thread::EchoThread) -> Self {
        Self {
            kind: ECHO_VALUE_THREAD,
            payload: value as u64,
        }
    }

    pub fn list(value: *mut EchoList) -> Self {
        Self {
            kind: ECHO_VALUE_LIST,
            payload: value as u64,
        }
    }

    pub fn array(value: *mut EchoArray) -> Self {
        Self {
            kind: ECHO_VALUE_ARRAY,
            payload: value as u64,
        }
    }

    pub fn object(value: *mut EchoObject) -> Self {
        Self {
            kind: ECHO_VALUE_OBJECT,
            payload: value as u64,
        }
    }

    pub const fn pending() -> Self {
        Self {
            kind: ECHO_VALUE_PENDING,
            payload: 0,
        }
    }

    pub fn tcp_listener(value: *mut net::EchoTcpListener) -> Self {
        Self {
            kind: ECHO_VALUE_TCP_LISTENER,
            payload: value as u64,
        }
    }

    pub fn tcp_connection(value: *mut net::EchoTcpConnection) -> Self {
        Self {
            kind: ECHO_VALUE_TCP_CONNECTION,
            payload: value as u64,
        }
    }

    pub(crate) fn string_bytes(self) -> Option<Vec<u8>> {
        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(Vec::new()),
            ECHO_VALUE_BOOL => {
                if self.payload == 0 {
                    Some(Vec::new())
                } else {
                    Some(b"1".to_vec())
                }
            }
            ECHO_VALUE_INT => Some((self.payload as i64).to_string().into_bytes()),
            ECHO_VALUE_FLOAT => Some(format_php_float(f64::from_bits(self.payload)).into_bytes()),
            ECHO_VALUE_STRING => unsafe {
                (self.payload as *const EchoString)
                    .as_ref()
                    .map(|value| value.bytes.clone())
            },
            ECHO_VALUE_ARRAY => Some(b"Array".to_vec()),
            ECHO_VALUE_LIST => Some(b"List".to_vec()),
            ECHO_VALUE_TASK
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_OBJECT
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_THREAD => Some(b"Object".to_vec()),
            _ => None,
        }
    }

    pub(crate) fn inspect_bytes(self) -> Option<Vec<u8>> {
        Some(self.inspect_string(0).into_bytes())
    }

    fn inspect_string(self, depth: usize) -> String {
        if depth >= INSPECT_MAX_DEPTH {
            return match self.kind {
                ECHO_VALUE_ARRAY => "Array [...]".to_string(),
                ECHO_VALUE_LIST => "List [...]".to_string(),
                ECHO_VALUE_OBJECT => "Object {...}".to_string(),
                _ => self
                    .string_bytes()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or_default(),
            };
        }

        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => String::new(),
            ECHO_VALUE_BOOL => {
                if self.payload == 0 {
                    String::new()
                } else {
                    "1".to_string()
                }
            }
            ECHO_VALUE_INT => (self.payload as i64).to_string(),
            ECHO_VALUE_FLOAT => format_php_float(f64::from_bits(self.payload)),
            ECHO_VALUE_STRING => unsafe {
                (self.payload as *const EchoString)
                    .as_ref()
                    .map(|value| {
                        if depth == 0 {
                            String::from_utf8_lossy(&value.bytes).into_owned()
                        } else {
                            inspect_string_literal(&value.bytes)
                        }
                    })
                    .unwrap_or_default()
            },
            ECHO_VALUE_ARRAY => unsafe {
                (self.payload as *const EchoArray)
                    .as_ref()
                    .map(|array| inspect_array(array, depth + 1))
                    .unwrap_or_else(|| "Array".to_string())
            },
            ECHO_VALUE_LIST => unsafe {
                (self.payload as *const EchoList)
                    .as_ref()
                    .map(|list| inspect_list(list, depth + 1))
                    .unwrap_or_else(|| "List".to_string())
            },
            ECHO_VALUE_TASK
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_OBJECT
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_THREAD => "Object".to_string(),
            _ => String::new(),
        }
    }

    fn int_value(self) -> Option<i64> {
        match self.kind {
            ECHO_VALUE_BOOL => Some(if self.payload == 0 { 0 } else { 1 }),
            ECHO_VALUE_INT => Some(self.payload as i64),
            ECHO_VALUE_FLOAT => Some(f64::from_bits(self.payload) as i64),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(self.payload as *const EchoString).as_ref()?.bytes;
                let text = std::str::from_utf8(bytes).ok()?.trim_ascii();
                text.parse::<i64>().ok()
            },
            _ => None,
        }
    }

    fn bool_value(self) -> Option<bool> {
        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(false),
            ECHO_VALUE_BOOL => Some(self.payload != 0),
            ECHO_VALUE_INT => Some(self.payload as i64 != 0),
            ECHO_VALUE_FLOAT => Some(f64::from_bits(self.payload) != 0.0),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(self.payload as *const EchoString).as_ref()?.bytes;
                Some(!bytes.is_empty() && bytes != b"0")
            },
            ECHO_VALUE_ARRAY | ECHO_VALUE_LIST => Some(true),
            ECHO_VALUE_TASK
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_TCP_LISTENER
            | ECHO_VALUE_TCP_CONNECTION
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_THREAD => Some(true),
            ECHO_VALUE_PENDING => Some(false),
            _ => None,
        }
    }

    fn php_int_value(self) -> Option<i64> {
        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(0),
            ECHO_VALUE_BOOL => Some(if self.payload == 0 { 0 } else { 1 }),
            ECHO_VALUE_INT => Some(self.payload as i64),
            ECHO_VALUE_FLOAT => Some(f64::from_bits(self.payload) as i64),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(self.payload as *const EchoString).as_ref()?.bytes;
                Some(parse_php_decimal_int(bytes))
            },
            _ => None,
        }
    }

    pub const fn is_string(self) -> bool {
        self.kind == ECHO_VALUE_STRING
    }

    pub const fn is_array(self) -> bool {
        self.kind == ECHO_VALUE_ARRAY
    }

    pub const fn is_list(self) -> bool {
        self.kind == ECHO_VALUE_LIST
    }

    pub const fn is_object(self) -> bool {
        self.kind == ECHO_VALUE_OBJECT
    }

    pub const fn is_task(self) -> bool {
        self.kind == ECHO_VALUE_TASK
    }

    pub const fn is_pending(self) -> bool {
        self.kind == ECHO_VALUE_PENDING
    }

    pub(crate) const fn is_true_bool(self) -> bool {
        self.kind == ECHO_VALUE_BOOL && self.payload != 0
    }

    pub(crate) fn type_name_bytes(self) -> &'static [u8] {
        match self.kind {
            ECHO_VALUE_NULL => b"null".as_slice(),
            ECHO_VALUE_BOOL => b"bool".as_slice(),
            ECHO_VALUE_INT => b"int".as_slice(),
            ECHO_VALUE_FLOAT => b"float".as_slice(),
            ECHO_VALUE_STRING => b"string".as_slice(),
            ECHO_VALUE_ARRAY => b"array".as_slice(),
            ECHO_VALUE_LIST => b"list".as_slice(),
            ECHO_VALUE_TASK => b"task".as_slice(),
            ECHO_VALUE_TASK_GROUP => b"task_group".as_slice(),
            ECHO_VALUE_THREAD => b"thread".as_slice(),
            ECHO_VALUE_PROCESS => b"process".as_slice(),
            ECHO_VALUE_PENDING => b"pending".as_slice(),
            ECHO_VALUE_TCP_LISTENER => b"TcpServer".as_slice(),
            ECHO_VALUE_TCP_CONNECTION => b"TcpConnection".as_slice(),
            ECHO_VALUE_OBJECT => b"object".as_slice(),
            _ => b"unknown".as_slice(),
        }
    }

    pub(crate) fn as_task_mut(self) -> Option<&'static mut task::EchoTask> {
        if self.kind != ECHO_VALUE_TASK || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut task::EchoTask).as_mut() }
    }

    pub(crate) fn as_task_group_mut(self) -> Option<&'static mut task_group::EchoTaskGroup> {
        if self.kind != ECHO_VALUE_TASK_GROUP || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut task_group::EchoTaskGroup).as_mut() }
    }

    pub(crate) fn as_process_mut(self) -> Option<&'static mut process::EchoProcess> {
        if self.kind != ECHO_VALUE_PROCESS || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut process::EchoProcess).as_mut() }
    }

    pub(crate) fn as_thread_mut(self) -> Option<&'static mut thread::EchoThread> {
        if self.kind != ECHO_VALUE_THREAD || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut thread::EchoThread).as_mut() }
    }

    pub(crate) fn as_tcp_listener_ref(self) -> Option<&'static net::EchoTcpListener> {
        if self.kind != ECHO_VALUE_TCP_LISTENER || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *const net::EchoTcpListener).as_ref() }
    }

    pub(crate) fn as_tcp_connection_mut(self) -> Option<&'static mut net::EchoTcpConnection> {
        if self.kind != ECHO_VALUE_TCP_CONNECTION || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut net::EchoTcpConnection).as_mut() }
    }
}

pub fn echo_is_callable(value: EchoValue) -> bool {
    echo_normalize_callable(value).is_ok_and(|callback| callback.is_some())
}

pub fn reset_execution_state() {
    reset_output_runtime();
    execution::reset();
    assertions::reset();
}

pub fn capture_stdout<T>(repl_inspect: bool, f: impl FnOnce() -> T) -> (T, Vec<u8>) {
    reset_execution_state();
    execution::begin_capture(repl_inspect);
    let result = f();
    (result, execution::finish_capture())
}

fn inspect_array(array: &EchoArray, depth: usize) -> String {
    let mut parts = Vec::new();
    for (key, value) in array
        .keys
        .iter()
        .zip(array.values.iter())
        .take(INSPECT_MAX_ITEMS)
    {
        parts.push(format!(
            "{} => {}",
            inspect_array_key(key),
            value.inspect_string(depth)
        ));
    }

    if array.values.len() > INSPECT_MAX_ITEMS {
        parts.push(format!(
            "... {} more",
            array.values.len() - INSPECT_MAX_ITEMS
        ));
    }

    format!("Array [{}]", parts.join(", "))
}

fn inspect_list(list: &EchoList, depth: usize) -> String {
    let mut parts = Vec::new();
    for value in list.values.iter().take(INSPECT_MAX_ITEMS) {
        parts.push(value.inspect_string(depth));
    }

    if list.values.len() > INSPECT_MAX_ITEMS {
        parts.push(format!(
            "... {} more",
            list.values.len() - INSPECT_MAX_ITEMS
        ));
    }

    format!("List [{}]", parts.join(", "))
}

fn inspect_array_key(key: &EchoArrayKey) -> String {
    match key {
        EchoArrayKey::Int(value) => value.to_string(),
        EchoArrayKey::String(bytes) => inspect_string_literal(bytes),
    }
}

fn inspect_string_literal(bytes: &[u8]) -> String {
    let mut literal = String::from("\"");
    for byte in bytes {
        match byte {
            b'\\' => literal.push_str("\\\\"),
            b'"' => literal.push_str("\\\""),
            b'\n' => literal.push_str("\\n"),
            b'\r' => literal.push_str("\\r"),
            b'\t' => literal.push_str("\\t"),
            0x20..=0x7e => literal.push(*byte as char),
            _ => literal.push_str(&format!("\\x{byte:02x}")),
        }
    }
    literal.push('"');
    literal
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_add(left: EchoValue, right: EchoValue) -> EchoValue {
    if left.is_array() || right.is_array() {
        return php_array_union(left, right);
    }

    php_numeric_binary(
        left,
        right,
        |left, right| left + right,
        |left, right| left + right,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_sub(left: EchoValue, right: EchoValue) -> EchoValue {
    php_numeric_binary(
        left,
        right,
        |left, right| left - right,
        |left, right| left - right,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_mul(left: EchoValue, right: EchoValue) -> EchoValue {
    php_numeric_binary(
        left,
        right,
        |left, right| left * right,
        |left, right| left * right,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_div(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = PhpNumber::coerce(left) else {
        return EchoValue::error();
    };
    let Some(right) = PhpNumber::coerce(right) else {
        return EchoValue::error();
    };

    match (left, right) {
        (_, PhpNumber::Int(0)) | (_, PhpNumber::Float(0.0)) => EchoValue::error(),
        (PhpNumber::Int(left), PhpNumber::Int(right)) if left % right == 0 => {
            EchoValue::int(left / right)
        }
        _ => EchoValue::float(left.as_float() / right.as_float()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_mod(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = left.php_int_value() else {
        return EchoValue::error();
    };
    let Some(right) = right.php_int_value() else {
        return EchoValue::error();
    };
    if right == 0 {
        return EchoValue::error();
    }

    EchoValue::int(left % right)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_pow(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = PhpNumber::coerce(left) else {
        return EchoValue::error();
    };
    let Some(right) = PhpNumber::coerce(right) else {
        return EchoValue::error();
    };

    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) if right >= 0 => {
            match u32::try_from(right)
                .ok()
                .and_then(|right| left.checked_pow(right))
            {
                Some(value) => EchoValue::int(value),
                None => EchoValue::float(pow_f64_int(left as f64, right)),
            }
        }
        (left, PhpNumber::Int(right)) => EchoValue::float(pow_f64_int(left.as_float(), right)),
        (left, PhpNumber::Float(right)) if right.fract() == 0.0 => {
            EchoValue::float(pow_f64_int(left.as_float(), right as i64))
        }
        (left, PhpNumber::Float(right)) => EchoValue::float(echo_math_pow(left.as_float(), right)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_unary_plus(value: EchoValue) -> EchoValue {
    match PhpNumber::coerce(value) {
        Some(PhpNumber::Int(value)) => EchoValue::int(value),
        Some(PhpNumber::Float(value)) => EchoValue::float(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_unary_minus(value: EchoValue) -> EchoValue {
    match PhpNumber::coerce(value) {
        Some(PhpNumber::Int(value)) => EchoValue::int(value.saturating_neg()),
        Some(PhpNumber::Float(value)) => EchoValue::float(-value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_bool(value: EchoValue) -> bool {
    value.bool_value().unwrap_or(false)
}

pub(crate) fn echo_runtime_string(bytes: Vec<u8>) -> EchoValue {
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_abs(value: EchoValue) -> EchoValue {
    match value.kind {
        ECHO_VALUE_INT => match (value.payload as i64).checked_abs() {
            Some(value) => EchoValue::int(value),
            None => EchoValue::error(),
        },
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_numeric(value: EchoValue) -> EchoValue {
    let is_numeric = match value.kind {
        ECHO_VALUE_INT => true,
        ECHO_VALUE_STRING => unsafe {
            (value.payload as *const EchoString)
                .as_ref()
                .is_some_and(|value| is_php_numeric_string(&value.bytes))
        },
        _ => false,
    };
    EchoValue::bool(is_numeric)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_bindec(value: EchoValue) -> EchoValue {
    php_string_to_number_builtin(value, |bytes| php_unsigned_base_to_decimal(bytes, 2))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hexdec(value: EchoValue) -> EchoValue {
    php_string_to_number_builtin(value, |bytes| php_unsigned_base_to_decimal(bytes, 16))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_octdec(value: EchoValue) -> EchoValue {
    php_string_to_number_builtin(value, |bytes| php_unsigned_base_to_decimal(bytes, 8))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_base_convert(
    value: EchoValue,
    from_base: EchoValue,
    to_base: EchoValue,
) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(from_base) = from_base.php_int_value() else {
        return EchoValue::error();
    };
    let Some(to_base) = to_base.php_int_value() else {
        return EchoValue::error();
    };
    if !(2..=36).contains(&from_base) || !(2..=36).contains(&to_base) {
        return EchoValue::error();
    }

    let Some(value) = php_unsigned_base_to_u128(&bytes, from_base as u32) else {
        return EchoValue::error();
    };

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        u128_to_base_bytes(value, to_base as u32),
    ))))
}

fn php_unsigned_base_to_decimal(bytes: &[u8], base: u32) -> EchoValue {
    let mut integer = 0u64;
    let mut float = 0.0;
    let mut overflowed = false;

    for digit in bytes
        .iter()
        .copied()
        .filter_map(|byte| ascii_digit_value(byte))
    {
        if digit >= base {
            continue;
        }

        if overflowed {
            float = float * base as f64 + digit as f64;
            continue;
        }

        match integer
            .checked_mul(base as u64)
            .and_then(|value| value.checked_add(digit as u64))
        {
            Some(value) => integer = value,
            None => {
                overflowed = true;
                float = integer as f64 * base as f64 + digit as f64;
            }
        }
    }

    if overflowed || integer > i64::MAX as u64 {
        EchoValue::float(if overflowed { float } else { integer as f64 })
    } else {
        EchoValue::int(integer as i64)
    }
}

fn php_unsigned_base_to_u128(bytes: &[u8], base: u32) -> Option<u128> {
    let mut value = 0u128;
    for digit in bytes
        .iter()
        .copied()
        .filter_map(|byte| ascii_digit_value(byte))
    {
        if digit >= base {
            continue;
        }

        value = value
            .checked_mul(base as u128)?
            .checked_add(digit as u128)?;
    }

    Some(value)
}

fn u128_to_base_bytes(mut value: u128, base: u32) -> Vec<u8> {
    if value == 0 {
        return b"0".to_vec();
    }

    let mut bytes = Vec::new();
    while value > 0 {
        let digit = (value % base as u128) as u8;
        bytes.push(match digit {
            0..=9 => b'0' + digit,
            _ => b'a' + (digit - 10),
        });
        value /= base as u128;
    }
    bytes.reverse();
    bytes
}

fn ascii_digit_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some((byte - b'0') as u32),
        b'a'..=b'z' => Some((byte - b'a' + 10) as u32),
        b'A'..=b'Z' => Some((byte - b'A' + 10) as u32),
        _ => None,
    }
}

fn parse_php_decimal_int(bytes: &[u8]) -> i64 {
    let bytes = trim_ascii_start(bytes);
    let (negative, digits) = match bytes.first().copied() {
        Some(b'-') => (true, &bytes[1..]),
        Some(b'+') => (false, &bytes[1..]),
        _ => (false, bytes),
    };

    let mut value = 0i64;
    let mut found_digit = false;
    for byte in digits.iter().copied() {
        if !byte.is_ascii_digit() {
            break;
        }
        found_digit = true;
        value = value
            .saturating_mul(10)
            .saturating_add((byte - b'0') as i64);
    }

    if !found_digit {
        return 0;
    }

    if negative {
        value.saturating_neg()
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PhpNumber {
    Int(i64),
    Float(f64),
}

impl PhpNumber {
    pub(crate) fn coerce(value: EchoValue) -> Option<Self> {
        match value.kind {
            ECHO_VALUE_NULL => Some(Self::Int(0)),
            ECHO_VALUE_BOOL => Some(Self::Int(if value.payload == 0 { 0 } else { 1 })),
            ECHO_VALUE_INT => Some(Self::Int(value.payload as i64)),
            ECHO_VALUE_FLOAT => Some(Self::Float(f64::from_bits(value.payload))),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(value.payload as *const EchoString).as_ref()?.bytes;
                parse_php_number(bytes)
            },
            _ => None,
        }
    }

    pub(crate) const fn as_float(self) -> f64 {
        match self {
            Self::Int(value) => value as f64,
            Self::Float(value) => value,
        }
    }

    pub(crate) fn into_echo_value(self) -> EchoValue {
        match self {
            Self::Int(value) => EchoValue::int(value),
            Self::Float(value) => EchoValue::float(value),
        }
    }
}

pub(crate) fn php_number_add(left: PhpNumber, right: PhpNumber) -> PhpNumber {
    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) => left
            .checked_add(right)
            .map(PhpNumber::Int)
            .unwrap_or_else(|| PhpNumber::Float(left as f64 + right as f64)),
        _ => PhpNumber::Float(left.as_float() + right.as_float()),
    }
}

pub(crate) fn php_number_mul(left: PhpNumber, right: PhpNumber) -> PhpNumber {
    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) => left
            .checked_mul(right)
            .map(PhpNumber::Int)
            .unwrap_or_else(|| PhpNumber::Float(left as f64 * right as f64)),
        _ => PhpNumber::Float(left.as_float() * right.as_float()),
    }
}

fn php_numeric_binary(
    left: EchoValue,
    right: EchoValue,
    int_op: impl FnOnce(i64, i64) -> i64,
    float_op: impl FnOnce(f64, f64) -> f64,
) -> EchoValue {
    let Some(left) = PhpNumber::coerce(left) else {
        return EchoValue::error();
    };
    let Some(right) = PhpNumber::coerce(right) else {
        return EchoValue::error();
    };

    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) => EchoValue::int(int_op(left, right)),
        _ => EchoValue::float(float_op(left.as_float(), right.as_float())),
    }
}

fn parse_php_number(bytes: &[u8]) -> Option<PhpNumber> {
    let bytes = trim_ascii(bytes);
    if bytes.is_empty() {
        return None;
    }

    let text = std::str::from_utf8(bytes).ok()?;
    if text.contains(['.', 'e', 'E']) {
        text.parse::<f64>().ok().map(PhpNumber::Float)
    } else {
        text.parse::<i64>().ok().map(PhpNumber::Int)
    }
}

fn pow_f64_int(base: f64, exponent: i64) -> f64 {
    if exponent == 0 {
        return 1.0;
    }

    let negative = exponent < 0;
    let mut exponent = exponent.unsigned_abs();
    let mut factor = base;
    let mut value = 1.0;

    while exponent > 0 {
        if exponent & 1 == 1 {
            value *= factor;
        }
        exponent >>= 1;
        factor *= factor;
    }

    if negative { 1.0 / value } else { value }
}

fn is_php_numeric_string(bytes: &[u8]) -> bool {
    let bytes = trim_ascii(bytes);
    if bytes.is_empty() {
        return false;
    }

    let mut index = match bytes.first().copied() {
        Some(b'-' | b'+') => 1,
        _ => 0,
    };

    let integer_digits = consume_ascii_digits(bytes, &mut index);
    let fraction_digits = if bytes.get(index) == Some(&b'.') {
        index += 1;
        consume_ascii_digits(bytes, &mut index)
    } else {
        0
    };

    if integer_digits + fraction_digits == 0 {
        return false;
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'-' | b'+')) {
            index += 1;
        }
        if consume_ascii_digits(bytes, &mut index) == 0 {
            return false;
        }
    }

    index == bytes.len()
}

fn consume_ascii_digits(bytes: &[u8], index: &mut usize) -> usize {
    let start = *index;
    while bytes.get(*index).is_some_and(u8::is_ascii_digit) {
        *index += 1;
    }
    *index - start
}

pub(crate) fn echo_values_equal(left: EchoValue, right: EchoValue) -> bool {
    if left.kind != right.kind {
        return false;
    }

    match left.kind {
        ECHO_VALUE_NULL => true,
        ECHO_VALUE_BOOL | ECHO_VALUE_INT | ECHO_VALUE_FLOAT => left.payload == right.payload,
        ECHO_VALUE_STRING => left.string_bytes() == right.string_bytes(),
        ECHO_VALUE_ARRAY => echo_arrays_equal(left, right, echo_values_equal),
        ECHO_VALUE_LIST => echo_lists_equal(left, right, echo_values_equal),
        _ => left.payload == right.payload,
    }
}

pub(crate) fn php_values_equal(left: EchoValue, right: EchoValue) -> bool {
    if let (Some(left), Some(right)) = (PhpNumber::coerce(left), PhpNumber::coerce(right)) {
        return left.as_float() == right.as_float();
    }

    match (left.string_bytes(), right.string_bytes()) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::{Duration, Instant};

    fn test_string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes.to_vec()))))
    }

    fn assert_float_value(value: EchoValue, expected: f64) {
        assert_eq!(value.kind, ECHO_VALUE_FLOAT);
        assert!((f64::from_bits(value.payload) - expected).abs() < 0.000000000001);
    }

    #[test]
    fn task_defer_returns_task_value() {
        unsafe extern "C" fn callback() -> EchoValue {
            EchoValue::int(1)
        }

        let value = echo_task_defer(Some(callback));

        assert!(value.is_task());
        assert_ne!(value.payload, 0);
    }

    #[test]
    fn task_run_starts_callback_before_join_collects_it() {
        static CALLBACK_RUNS: AtomicUsize = AtomicUsize::new(0);

        unsafe extern "C" fn callback() -> EchoValue {
            CALLBACK_RUNS.fetch_add(1, Ordering::Relaxed);
            EchoValue::int(42)
        }

        CALLBACK_RUNS.store(0, Ordering::Relaxed);
        let task = echo_task_defer(Some(callback));
        let task = echo_task_run(task);

        let deadline = Instant::now() + Duration::from_secs(1);
        while CALLBACK_RUNS.load(Ordering::Relaxed) == 0 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }

        assert_eq!(CALLBACK_RUNS.load(Ordering::Relaxed), 1);

        let result = echo_task_join(task);

        assert_eq!(CALLBACK_RUNS.load(Ordering::Relaxed), 1);
        assert_eq!(result, EchoValue::int(42));
    }

    #[test]
    fn integer_arithmetic_core_abi_adds_and_subtracts() {
        assert_eq!(
            echo_value_add(EchoValue::int(3), EchoValue::int(5)),
            EchoValue::int(8)
        );
        assert_eq!(
            echo_value_sub(EchoValue::int(3), EchoValue::int(5)),
            EchoValue::int(-2)
        );
        assert_eq!(
            echo_value_sub(EchoValue::int(3), test_string_value(b"not numeric")),
            EchoValue::error()
        );
    }

    #[test]
    fn php_numeric_arithmetic_coerces_strings_bools_and_null() {
        assert_float_value(
            echo_value_add(test_string_value(b"3.2"), test_string_value(b"3.4")),
            6.6,
        );
        assert_eq!(
            echo_value_add(EchoValue::null(), EchoValue::int(5)),
            EchoValue::int(5)
        );
        assert_eq!(
            echo_value_add(EchoValue::bool(true), EchoValue::int(2)),
            EchoValue::int(3)
        );
    }

    #[test]
    fn php_array_add_uses_union_semantics_for_numeric_keys() {
        let left = EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(vec![
            EchoValue::int(1),
            EchoValue::int(2),
        ]))));
        let right = EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(vec![
            EchoValue::int(3),
            EchoValue::int(4),
            EchoValue::int(5),
        ]))));

        let result = echo_value_add(left, right);
        let array = unsafe { (result.payload as *const EchoArray).as_ref() }.expect("array");

        assert_eq!(
            array.values,
            vec![EchoValue::int(1), EchoValue::int(2), EchoValue::int(5)]
        );
    }

    #[test]
    fn php_array_add_uses_union_semantics_for_string_keys() {
        let left_key = test_string_value(b"a");
        let duplicate_key = test_string_value(b"a");
        let new_key = test_string_value(b"b");
        let left = echo_value_array_set(echo_value_array_new(), left_key, EchoValue::int(1));
        let right = echo_value_array_set(echo_value_array_new(), duplicate_key, EchoValue::int(2));
        let right = echo_value_array_set(right, new_key, EchoValue::int(3));

        let result = echo_value_add(left, right);
        let array = unsafe { (result.payload as *const EchoArray).as_ref() }.expect("array");

        assert_eq!(array.keys.len(), 2);
        assert_eq!(array.values, vec![EchoValue::int(1), EchoValue::int(3)]);
        assert_eq!(echo_php_array_is_list(result), EchoValue::bool(false));
    }

    #[test]
    fn array_key_value_and_aggregate_builtins_preserve_php_array_behavior() {
        let id = test_string_value(b"id");
        let qty = test_string_value(b"qty");
        let string_two = test_string_value(b"2");
        let mut array = echo_value_array_new();
        array = echo_value_array_set(array, id, EchoValue::int(10));
        array = echo_value_array_set(array, qty, string_two);
        array = echo_value_array_set(array, EchoValue::int(5), EchoValue::int(2));

        let values = echo_php_array_values(array);
        let values_ref = unsafe { (values.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            values_ref.keys,
            vec![
                EchoArrayKey::Int(0),
                EchoArrayKey::Int(1),
                EchoArrayKey::Int(2)
            ]
        );
        assert_eq!(
            values_ref.values,
            vec![EchoValue::int(10), string_two, EchoValue::int(2)]
        );

        let keys = echo_php_array_keys(array, EchoValue::pending(), EchoValue::bool(false));
        let keys_ref = unsafe { (keys.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(keys_ref.values[0].string_bytes(), Some(b"id".to_vec()));
        assert_eq!(keys_ref.values[1].string_bytes(), Some(b"qty".to_vec()));
        assert_eq!(keys_ref.values[2], EchoValue::int(5));

        let loose = echo_php_array_keys(array, EchoValue::int(2), EchoValue::bool(false));
        let loose_ref = unsafe { (loose.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(loose_ref.values.len(), 2);
        assert_eq!(loose_ref.values[0].string_bytes(), Some(b"qty".to_vec()));
        assert_eq!(loose_ref.values[1], EchoValue::int(5));

        let strict = echo_php_array_keys(array, EchoValue::int(2), EchoValue::bool(true));
        let strict_ref = unsafe { (strict.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(strict_ref.values, vec![EchoValue::int(5)]);

        assert_eq!(echo_php_array_sum(array), EchoValue::int(14));
        assert_eq!(echo_php_array_product(array), EchoValue::int(40));
        assert_eq!(
            echo_php_array_sum(echo_value_array_new()),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_array_product(echo_value_array_new()),
            EchoValue::int(1)
        );
    }

    #[test]
    fn array_fill_builtins_preserve_php_key_construction_behavior() {
        let fill = echo_php_array_fill(
            EchoValue::int(-2),
            EchoValue::int(4),
            test_string_value(b"pear"),
        );
        let fill_ref = unsafe { (fill.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            fill_ref.keys,
            vec![
                EchoArrayKey::Int(-2),
                EchoArrayKey::Int(-1),
                EchoArrayKey::Int(0),
                EchoArrayKey::Int(1)
            ]
        );
        assert!(
            fill_ref
                .values
                .iter()
                .all(|value| value.string_bytes() == Some(b"pear".to_vec()))
        );
        assert_eq!(
            echo_php_array_fill(EchoValue::int(0), EchoValue::int(-1), EchoValue::null()).kind,
            ECHO_VALUE_ERROR
        );

        let mut keys = echo_value_array_new();
        keys = echo_value_array_append(keys, test_string_value(b"sku"));
        keys = echo_value_array_append(keys, test_string_value(b"2"));
        keys = echo_value_array_append(keys, EchoValue::int(5));
        keys = echo_value_array_append(keys, EchoValue::bool(true));
        keys = echo_value_array_append(keys, EchoValue::null());
        keys = echo_value_array_append(keys, test_string_value(b"sku"));
        let keyed = echo_php_array_fill_keys(keys, test_string_value(b"todo"));
        let keyed_ref = unsafe { (keyed.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            keyed_ref.keys,
            vec![
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::Int(2),
                EchoArrayKey::Int(5),
                EchoArrayKey::Int(1),
                EchoArrayKey::String(Vec::new())
            ]
        );
        assert!(
            keyed_ref
                .values
                .iter()
                .all(|value| value.string_bytes() == Some(b"todo".to_vec()))
        );
    }

    #[test]
    fn array_combine_and_pad_builtins_preserve_php_key_behavior() {
        let mut keys = echo_value_array_new();
        keys = echo_value_array_append(keys, test_string_value(b"sku"));
        keys = echo_value_array_append(keys, test_string_value(b"qty"));
        keys = echo_value_array_append(keys, test_string_value(b"qty"));
        keys = echo_value_array_append(keys, test_string_value(b"2"));

        let mut values = echo_value_array_new();
        values = echo_value_array_append(values, test_string_value(b"A-42"));
        values = echo_value_array_append(values, EchoValue::int(3));
        values = echo_value_array_append(values, EchoValue::int(4));
        values = echo_value_array_append(values, test_string_value(b"numeric"));

        let combined = echo_php_array_combine(keys, values);
        let combined_ref =
            unsafe { (combined.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            combined_ref.keys,
            vec![
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::String(b"qty".to_vec()),
                EchoArrayKey::Int(2),
            ]
        );
        assert_eq!(
            combined_ref.values[0].string_bytes(),
            Some(b"A-42".to_vec())
        );
        assert_eq!(combined_ref.values[1], EchoValue::int(4));
        assert_eq!(
            combined_ref.values[2].string_bytes(),
            Some(b"numeric".to_vec())
        );

        let mut row = echo_value_array_new();
        row = echo_value_array_set(row, test_string_value(b"sku"), test_string_value(b"A-42"));
        row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"seven"));
        row = echo_value_array_set(row, test_string_value(b"qty"), EchoValue::int(4));

        let right = echo_php_array_pad(row, EchoValue::int(5), test_string_value(b"missing"));
        let right_ref = unsafe { (right.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            right_ref.keys,
            vec![
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::Int(0),
                EchoArrayKey::String(b"qty".to_vec()),
                EchoArrayKey::Int(1),
                EchoArrayKey::Int(2),
            ]
        );

        let left = echo_php_array_pad(row, EchoValue::int(-5), test_string_value(b"missing"));
        let left_ref = unsafe { (left.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            left_ref.keys,
            vec![
                EchoArrayKey::Int(0),
                EchoArrayKey::Int(1),
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::Int(2),
                EchoArrayKey::String(b"qty".to_vec()),
            ]
        );

        let unchanged = echo_php_array_pad(row, EchoValue::int(2), test_string_value(b"noop"));
        let unchanged_ref =
            unsafe { (unchanged.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            unchanged_ref.keys,
            vec![
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::Int(7),
                EchoArrayKey::String(b"qty".to_vec()),
            ]
        );
    }

    #[test]
    fn array_slice_and_chunk_builtins_preserve_php_key_behavior() {
        let mut row = echo_value_array_new();
        row = echo_value_array_set(row, test_string_value(b"id"), EchoValue::int(101));
        row = echo_value_array_set(row, test_string_value(b"sku"), test_string_value(b"A-42"));
        row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"warehouse"));
        row = echo_value_array_set(
            row,
            test_string_value(b"status"),
            test_string_value(b"active"),
        );
        row = echo_value_array_set(row, EchoValue::int(8), test_string_value(b"late"));
        row = echo_value_array_set(row, test_string_value(b"owner"), test_string_value(b"maya"));

        let slice = echo_php_array_slice(
            row,
            EchoValue::int(1),
            EchoValue::int(-1),
            EchoValue::bool(false),
        );
        let slice_ref = unsafe { (slice.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            slice_ref.keys,
            vec![
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::Int(0),
                EchoArrayKey::String(b"status".to_vec()),
                EchoArrayKey::Int(1),
            ]
        );
        assert_eq!(slice_ref.values[0].string_bytes(), Some(b"A-42".to_vec()));
        assert_eq!(
            slice_ref.values[1].string_bytes(),
            Some(b"warehouse".to_vec())
        );
        assert_eq!(slice_ref.values[2].string_bytes(), Some(b"active".to_vec()));
        assert_eq!(slice_ref.values[3].string_bytes(), Some(b"late".to_vec()));

        let preserved = echo_php_array_slice(
            row,
            EchoValue::int(-4),
            EchoValue::int(3),
            EchoValue::bool(true),
        );
        let preserved_ref =
            unsafe { (preserved.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            preserved_ref.keys,
            vec![
                EchoArrayKey::Int(7),
                EchoArrayKey::String(b"status".to_vec()),
                EchoArrayKey::Int(8),
            ]
        );

        let chunks = echo_php_array_chunk(row, EchoValue::int(2), EchoValue::bool(false));
        let chunks_ref = unsafe { (chunks.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            chunks_ref.keys,
            vec![
                EchoArrayKey::Int(0),
                EchoArrayKey::Int(1),
                EchoArrayKey::Int(2)
            ]
        );
        let chunk_0 =
            unsafe { (chunks_ref.values[0].payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            chunk_0.keys,
            vec![EchoArrayKey::Int(0), EchoArrayKey::Int(1)]
        );
        assert_eq!(chunk_0.values[0], EchoValue::int(101));
        assert_eq!(chunk_0.values[1].string_bytes(), Some(b"A-42".to_vec()));
        let chunk_1 =
            unsafe { (chunks_ref.values[1].payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            chunk_1.keys,
            vec![EchoArrayKey::Int(0), EchoArrayKey::Int(1)]
        );
        assert_eq!(
            chunk_1.values[0].string_bytes(),
            Some(b"warehouse".to_vec())
        );
        assert_eq!(chunk_1.values[1].string_bytes(), Some(b"active".to_vec()));

        let preserved_chunks = echo_php_array_chunk(row, EchoValue::int(2), EchoValue::bool(true));
        let preserved_chunks_ref =
            unsafe { (preserved_chunks.payload as *const EchoArray).as_ref() }.expect("array");
        let preserved_chunk_1 =
            unsafe { (preserved_chunks_ref.values[1].payload as *const EchoArray).as_ref() }
                .expect("array");
        assert_eq!(
            preserved_chunk_1.keys,
            vec![
                EchoArrayKey::Int(7),
                EchoArrayKey::String(b"status".to_vec())
            ]
        );
        let preserved_chunk_2 =
            unsafe { (preserved_chunks_ref.values[2].payload as *const EchoArray).as_ref() }
                .expect("array");
        assert_eq!(
            preserved_chunk_2.values[0].string_bytes(),
            Some(b"late".to_vec())
        );
        assert_eq!(
            preserved_chunk_2.values[1].string_bytes(),
            Some(b"maya".to_vec())
        );

        assert_eq!(
            echo_php_array_chunk(row, EchoValue::int(0), EchoValue::bool(false)).kind,
            ECHO_VALUE_ERROR
        );
    }

    #[test]
    fn array_merge_and_replace_builtins_preserve_php_key_behavior() {
        let mut base = echo_value_array_new();
        base = echo_value_array_set(base, test_string_value(b"sku"), test_string_value(b"A-42"));
        base = echo_value_array_set(base, EchoValue::int(7), test_string_value(b"old-bin"));
        base = echo_value_array_set(
            base,
            test_string_value(b"status"),
            test_string_value(b"draft"),
        );

        let mut override_row = echo_value_array_new();
        override_row = echo_value_array_set(
            override_row,
            test_string_value(b"status"),
            test_string_value(b"active"),
        );
        override_row = echo_value_array_set(
            override_row,
            EchoValue::int(4),
            test_string_value(b"new-bin"),
        );
        override_row = echo_value_array_set(
            override_row,
            test_string_value(b"owner"),
            test_string_value(b"maya"),
        );

        let mut extra = echo_value_array_new();
        extra = echo_value_array_set(extra, test_string_value(b"sku"), test_string_value(b"A-43"));
        extra = echo_value_array_set(extra, EchoValue::int(9), test_string_value(b"late"));

        let mut args = echo_value_array_new();
        args = echo_value_array_append(args, base);
        args = echo_value_array_append(args, override_row);
        args = echo_value_array_append(args, extra);

        let merged = echo_php_array_merge(args);
        let merged_ref = unsafe { (merged.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            merged_ref.keys,
            vec![
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::Int(0),
                EchoArrayKey::String(b"status".to_vec()),
                EchoArrayKey::Int(1),
                EchoArrayKey::String(b"owner".to_vec()),
                EchoArrayKey::Int(2),
            ]
        );
        assert_eq!(merged_ref.values[0].string_bytes(), Some(b"A-43".to_vec()));
        assert_eq!(
            merged_ref.values[1].string_bytes(),
            Some(b"old-bin".to_vec())
        );
        assert_eq!(
            merged_ref.values[2].string_bytes(),
            Some(b"active".to_vec())
        );
        assert_eq!(
            merged_ref.values[3].string_bytes(),
            Some(b"new-bin".to_vec())
        );
        assert_eq!(merged_ref.values[4].string_bytes(), Some(b"maya".to_vec()));
        assert_eq!(merged_ref.values[5].string_bytes(), Some(b"late".to_vec()));

        let replaced = echo_php_array_replace(args);
        let replaced_ref =
            unsafe { (replaced.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            replaced_ref.keys,
            vec![
                EchoArrayKey::String(b"sku".to_vec()),
                EchoArrayKey::Int(7),
                EchoArrayKey::String(b"status".to_vec()),
                EchoArrayKey::Int(4),
                EchoArrayKey::String(b"owner".to_vec()),
                EchoArrayKey::Int(9),
            ]
        );
        assert_eq!(
            replaced_ref.values[0].string_bytes(),
            Some(b"A-43".to_vec())
        );
        assert_eq!(
            replaced_ref.values[1].string_bytes(),
            Some(b"old-bin".to_vec())
        );
        assert_eq!(
            replaced_ref.values[2].string_bytes(),
            Some(b"active".to_vec())
        );

        let empty = echo_php_array_merge(echo_value_array_new());
        let empty_ref = unsafe { (empty.payload as *const EchoArray).as_ref() }.expect("array");
        assert!(empty_ref.keys.is_empty());
        assert_eq!(
            echo_php_array_replace(echo_value_array_new()).kind,
            ECHO_VALUE_ERROR
        );
    }

    #[test]
    fn array_order_builtins_preserve_php_key_behavior() {
        let sku = test_string_value(b"sku");
        let qty = test_string_value(b"qty");
        let mut row = echo_value_array_new();
        row = echo_value_array_set(row, sku, test_string_value(b"A-42"));
        row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"seven"));
        row = echo_value_array_set(row, qty, test_string_value(b"2"));
        row = echo_value_array_set(row, EchoValue::int(10), test_string_value(b"ten"));

        let reversed = echo_php_array_reverse(row, EchoValue::bool(false));
        let reversed_ref =
            unsafe { (reversed.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            reversed_ref.keys,
            vec![
                EchoArrayKey::Int(0),
                EchoArrayKey::String(b"qty".to_vec()),
                EchoArrayKey::Int(1),
                EchoArrayKey::String(b"sku".to_vec())
            ]
        );
        assert_eq!(
            reversed_ref
                .values
                .iter()
                .map(|value| value.string_bytes().unwrap())
                .collect::<Vec<_>>(),
            vec![
                b"ten".to_vec(),
                b"2".to_vec(),
                b"seven".to_vec(),
                b"A-42".to_vec()
            ]
        );

        let preserved = echo_php_array_reverse(row, EchoValue::bool(true));
        let preserved_ref =
            unsafe { (preserved.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            preserved_ref.keys,
            vec![
                EchoArrayKey::Int(10),
                EchoArrayKey::String(b"qty".to_vec()),
                EchoArrayKey::Int(7),
                EchoArrayKey::String(b"sku".to_vec())
            ]
        );

        let mut map = echo_value_array_new();
        map = echo_value_array_set(map, test_string_value(b"first"), test_string_value(b"id"));
        map = echo_value_array_set(map, test_string_value(b"second"), test_string_value(b"qty"));
        map = echo_value_array_set(map, test_string_value(b"third"), test_string_value(b"id"));
        map = echo_value_array_set(map, test_string_value(b"num"), test_string_value(b"2"));
        map = echo_value_array_set(map, test_string_value(b"int"), EchoValue::int(5));
        map = echo_value_array_set(map, test_string_value(b"skip"), EchoValue::bool(true));

        let flipped = echo_php_array_flip(map);
        let flipped_ref = unsafe { (flipped.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            flipped_ref.keys,
            vec![
                EchoArrayKey::String(b"id".to_vec()),
                EchoArrayKey::String(b"qty".to_vec()),
                EchoArrayKey::Int(2),
                EchoArrayKey::Int(5)
            ]
        );
        assert_eq!(
            flipped_ref.values[0].string_bytes(),
            Some(b"third".to_vec())
        );
        assert_eq!(
            flipped_ref.values[1].string_bytes(),
            Some(b"second".to_vec())
        );
        assert_eq!(flipped_ref.values[2].string_bytes(), Some(b"num".to_vec()));
        assert_eq!(flipped_ref.values[3].string_bytes(), Some(b"int".to_vec()));
    }

    #[test]
    fn array_search_and_count_values_preserve_php_value_behavior() {
        let mut row = echo_value_array_new();
        row = echo_value_array_set(row, test_string_value(b"sku"), test_string_value(b"A-42"));
        row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"A-42"));
        row = echo_value_array_set(row, test_string_value(b"qty"), EchoValue::int(4));
        row = echo_value_array_set(row, test_string_value(b"flag"), EchoValue::bool(true));
        row = echo_value_array_set(row, test_string_value(b"code"), test_string_value(b"4"));

        assert_eq!(
            echo_php_array_search(EchoValue::int(4), row, EchoValue::bool(false)).string_bytes(),
            Some(b"qty".to_vec())
        );
        assert_eq!(
            echo_php_array_search(EchoValue::int(4), row, EchoValue::bool(true)).string_bytes(),
            Some(b"qty".to_vec())
        );
        assert_eq!(
            echo_php_array_search(test_string_value(b"A-42"), row, EchoValue::bool(true))
                .string_bytes(),
            Some(b"sku".to_vec())
        );
        assert_eq!(
            echo_php_array_search(test_string_value(b"missing"), row, EchoValue::bool(true)),
            EchoValue::bool(false)
        );

        let mut values = echo_value_array_new();
        values = echo_value_array_append(values, test_string_value(b"new"));
        values = echo_value_array_append(values, test_string_value(b"new"));
        values = echo_value_array_append(values, test_string_value(b"done"));
        values = echo_value_array_append(values, EchoValue::int(2));
        values = echo_value_array_append(values, test_string_value(b"2"));
        values = echo_value_array_append(values, EchoValue::int(3));
        values = echo_value_array_append(values, EchoValue::bool(true));

        let counts = echo_php_array_count_values(values);
        let counts_ref = unsafe { (counts.payload as *const EchoArray).as_ref() }.expect("array");
        assert_eq!(
            counts_ref.keys,
            vec![
                EchoArrayKey::String(b"new".to_vec()),
                EchoArrayKey::String(b"done".to_vec()),
                EchoArrayKey::Int(2),
                EchoArrayKey::Int(3),
            ]
        );
        assert_eq!(
            counts_ref.values,
            vec![
                EchoValue::int(2),
                EchoValue::int(1),
                EchoValue::int(2),
                EchoValue::int(1)
            ]
        );
    }

    #[test]
    fn array_lookup_builtins_preserve_php_key_and_value_behavior() {
        let id = test_string_value(b"id");
        let qty = test_string_value(b"qty");
        let string_two = test_string_value(b"2");
        let zero = test_string_value(b"0");
        let mut array = echo_value_array_new();
        array = echo_value_array_set(array, id, EchoValue::int(10));
        array = echo_value_array_set(array, qty, string_two);
        array = echo_value_array_set(array, EchoValue::int(5), EchoValue::null());
        array = echo_value_array_set(array, zero, test_string_value(b"zero"));

        assert_eq!(
            echo_php_array_key_exists(test_string_value(b"id"), array),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_array_key_exists(EchoValue::int(5), array),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_array_key_exists(test_string_value(b"missing"), array),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_array_key_exists(EchoValue::bool(false), array),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_array_key_first(array).string_bytes(),
            Some(b"id".to_vec())
        );
        assert_eq!(echo_php_array_key_last(array), EchoValue::int(0));
        assert_eq!(
            echo_php_array_key_first(echo_value_array_new()),
            EchoValue::null()
        );
        assert_eq!(
            echo_php_array_key_last(echo_value_array_new()),
            EchoValue::null()
        );
        assert_eq!(
            echo_php_in_array(EchoValue::int(2), array, EchoValue::bool(false)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_in_array(EchoValue::int(2), array, EchoValue::bool(true)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_in_array(string_two, array, EchoValue::bool(true)),
            EchoValue::bool(true)
        );
    }

    #[test]
    fn repl_inspector_displays_array_values() {
        let key = test_string_value(b"name");
        let array = echo_value_array_set(echo_value_array_new(), key, test_string_value(b"Echo"));
        let array = echo_value_array_append(array, EchoValue::int(4));

        assert_eq!(
            array.inspect_bytes(),
            Some(br#"Array ["name" => "Echo", 0 => 4]"#.to_vec())
        );
    }

    #[test]
    fn repl_inspector_truncates_large_arrays() {
        let mut array = echo_value_array_new();
        for value in 0..10 {
            array = echo_value_array_append(array, EchoValue::int(value));
        }

        assert_eq!(
            array.inspect_bytes(),
            Some(b"Array [0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7, ... 2 more]".to_vec())
        );
    }

    #[test]
    fn runtime_capture_stdout_enables_repl_inspection_without_process_env() {
        let array = echo_value_array_append(echo_value_array_new(), EchoValue::int(2));

        let ((), stdout) = capture_stdout(true, || unsafe {
            echo_write_value(array);
        });

        assert_eq!(stdout, b"Array [0 => 2]");
    }

    #[test]
    fn index_get_reads_array_and_list_values() {
        let array = echo_value_array_append(echo_value_array_new(), EchoValue::int(4));
        let key = test_string_value(b"name");
        let array = echo_value_array_set(array, key, test_string_value(b"Echo"));

        assert_eq!(
            echo_value_index_get(array, EchoValue::int(0)),
            EchoValue::int(4)
        );
        assert_eq!(
            echo_value_index_get(array, test_string_value(b"name")).string_bytes(),
            Some(b"Echo".to_vec())
        );

        let list = echo_value_list_append(echo_value_list_new(), EchoValue::int(7));
        assert_eq!(
            echo_value_index_get(list, EchoValue::int(0)),
            EchoValue::int(7)
        );
    }

    #[test]
    fn index_get_returns_null_for_missing_values() {
        assert_eq!(
            echo_value_index_get(echo_value_array_new(), EchoValue::int(0)),
            EchoValue::null()
        );
        assert_eq!(
            echo_value_index_get(echo_value_list_new(), EchoValue::int(0)),
            EchoValue::null()
        );
    }

    #[test]
    fn php_arithmetic_core_abi_handles_remaining_operators() {
        assert_eq!(
            echo_value_mul(EchoValue::int(3), EchoValue::int(5)),
            EchoValue::int(15)
        );
        assert_eq!(
            echo_value_div(EchoValue::int(5), EchoValue::int(2)),
            EchoValue::float(2.5)
        );
        assert_eq!(
            echo_value_div(EchoValue::int(6), EchoValue::int(3)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_value_mod(EchoValue::int(-5), EchoValue::int(3)),
            EchoValue::int(-2)
        );
        assert_eq!(
            echo_value_pow(EchoValue::int(2), EchoValue::int(3)),
            EchoValue::int(8)
        );
        assert_eq!(
            echo_value_unary_minus(EchoValue::float(2.5)),
            EchoValue::float(-2.5)
        );
    }

    #[test]
    fn std_net_abi_exchanges_loopback_bytes() {
        let address = unsafe { echo_value_string(c"127.0.0.1:0".as_ptr().cast(), 11) };
        let server = echo_std_net_listen(address);
        assert_eq!(server.kind, ECHO_VALUE_TCP_LISTENER);

        let listener = server.as_tcp_listener_ref().expect("listener");
        let address = listener.local_addr().expect("local addr").to_string();

        let client = thread::spawn(move || {
            let address = unsafe { echo_value_string(address.as_ptr(), address.len()) };
            let connection = echo_std_net_connect(address);
            assert_eq!(connection.kind, ECHO_VALUE_TCP_CONNECTION);

            let request = unsafe { echo_value_string(c"ping".as_ptr().cast(), 4) };
            assert_eq!(echo_std_net_write(connection, request), EchoValue::int(4));
            let response = echo_std_net_read(connection, EchoValue::int(4));
            assert_eq!(response.string_bytes().expect("response"), b"pong");
            assert_eq!(echo_std_net_close(connection), EchoValue::null());
        });

        let connection = echo_std_net_accept(server);
        assert_eq!(connection.kind, ECHO_VALUE_TCP_CONNECTION);
        let request = echo_std_net_read(connection, EchoValue::int(4));
        assert_eq!(request.string_bytes().expect("request"), b"ping");
        let response = unsafe { echo_value_string(c"pong".as_ptr().cast(), 4) };
        assert_eq!(echo_std_net_write(connection, response), EchoValue::int(4));
        assert_eq!(echo_std_net_close(connection), EchoValue::null());

        client.join().expect("client");
    }

    #[test]
    fn std_http_response_text_formats_http_response() {
        let body = unsafe { echo_value_string(c"hello".as_ptr().cast(), 5) };
        let response = echo_std_http_response_text(body);

        assert_eq!(
            response.string_bytes().expect("response"),
            b"HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ncontent-length: 5\r\nconnection: close\r\n\r\nhello"
        );
    }

    #[test]
    fn null_normalizes_to_no_callable() {
        assert_eq!(echo_normalize_callable(EchoValue::null()), Ok(None));
        assert!(!echo_is_callable(EchoValue::null()));
    }

    #[test]
    fn invalid_value_does_not_normalize_to_callable() {
        let value = EchoValue {
            kind: 999,
            payload: 0,
        };

        assert_eq!(
            echo_normalize_callable(value),
            Err(EchoError::InvalidCallable)
        );
        assert!(!echo_is_callable(value));
    }

    #[test]
    fn string_value_normalizes_to_function_callable() {
        let string = Box::into_raw(Box::new(EchoString {
            bytes: b"filter".to_vec(),
        }));
        let value = EchoValue::string(string);

        assert_eq!(
            echo_normalize_callable(value),
            Ok(Some(EchoCallable::Function(EchoSymbol::new("filter"))))
        );
        assert!(echo_is_callable(value));

        unsafe {
            drop(Box::from_raw(string));
        }
    }

    #[test]
    fn non_utf8_string_value_is_not_callable() {
        let string = Box::into_raw(Box::new(EchoString { bytes: vec![0xff] }));
        let value = EchoValue::string(string);

        assert_eq!(
            echo_normalize_callable(value),
            Err(EchoError::InvalidCallable)
        );

        unsafe {
            drop(Box::from_raw(string));
        }
    }

    #[test]
    fn function_callable_call_fails_until_registry_exists() {
        let callable = EchoCallable::Function(EchoSymbol::new("filter"));

        assert_eq!(
            echo_call(&callable, &[]),
            Err(EchoError::UndefinedFunction(EchoSymbol::new("filter")))
        );
    }

    #[test]
    fn string_case_builtins_convert_only_ascii_bytes() {
        let string = Box::into_raw(Box::new(EchoString {
            bytes: "Echo äÖ 123!".as_bytes().to_vec(),
        }));
        let value = EchoValue::string(string);

        assert_eq!(
            echo_php_strtoupper(value).string_bytes(),
            Some("ECHO äÖ 123!".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strtolower(value).string_bytes(),
            Some("echo äÖ 123!".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(string));
        }
    }

    #[test]
    fn ucwords_preserves_php_default_separator_byte_behavior() {
        let words = Box::into_raw(Box::new(EchoString {
            bytes: "hello world".as_bytes().to_vec(),
        }));
        let tab = Box::into_raw(Box::new(EchoString {
            bytes: "hello\tworld".as_bytes().to_vec(),
        }));
        let hyphen = Box::into_raw(Box::new(EchoString {
            bytes: "hello-world".as_bytes().to_vec(),
        }));
        let mixed = Box::into_raw(Box::new(EchoString {
            bytes: "mIXed CASE".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "ächo world".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_ucwords(EchoValue::string(words)).string_bytes(),
            Some("Hello World".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_ucwords(EchoValue::string(tab)).string_bytes(),
            Some("Hello\tWorld".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_ucwords(EchoValue::string(hyphen)).string_bytes(),
            Some("Hello-world".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_ucwords(EchoValue::string(mixed)).string_bytes(),
            Some("MIXed CASE".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_ucwords(EchoValue::string(non_ascii)).string_bytes(),
            Some("ächo World".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(words));
            drop(Box::from_raw(tab));
            drop(Box::from_raw(hyphen));
            drop(Box::from_raw(mixed));
            drop(Box::from_raw(non_ascii));
        }
    }

    #[test]
    fn strval_preserves_php_scalar_string_coercion() {
        let string = Box::into_raw(Box::new(EchoString {
            bytes: "hello".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strval(EchoValue::string(string)).string_bytes(),
            Some("hello".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strval(EchoValue::int(42)).string_bytes(),
            Some("42".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strval(EchoValue::bool(true)).string_bytes(),
            Some("1".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strval(EchoValue::bool(false)).string_bytes(),
            Some(Vec::new())
        );
        assert_eq!(
            echo_php_strval(EchoValue::null()).string_bytes(),
            Some(Vec::new())
        );

        unsafe {
            drop(Box::from_raw(string));
        }
    }

    #[test]
    fn boolval_preserves_php_scalar_truthiness() {
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let zero = Box::into_raw(Box::new(EchoString {
            bytes: "0".as_bytes().to_vec(),
        }));
        let false_text = Box::into_raw(Box::new(EchoString {
            bytes: "false".as_bytes().to_vec(),
        }));

        assert_eq!(echo_php_boolval(EchoValue::null()), EchoValue::bool(false));
        assert_eq!(
            echo_php_boolval(EchoValue::bool(false)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_boolval(EchoValue::bool(true)),
            EchoValue::bool(true)
        );
        assert_eq!(echo_php_boolval(EchoValue::int(0)), EchoValue::bool(false));
        assert_eq!(echo_php_boolval(EchoValue::int(42)), EchoValue::bool(true));
        assert_eq!(
            echo_php_boolval(EchoValue::string(empty)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_boolval(EchoValue::string(zero)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_boolval(EchoValue::string(false_text)),
            EchoValue::bool(true)
        );

        unsafe {
            drop(Box::from_raw(empty));
            drop(Box::from_raw(zero));
            drop(Box::from_raw(false_text));
        }
    }

    #[test]
    fn intval_preserves_php_default_base_scalar_coercion() {
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let prefixed = Box::into_raw(Box::new(EchoString {
            bytes: "42abc".as_bytes().to_vec(),
        }));
        let spaced = Box::into_raw(Box::new(EchoString {
            bytes: "  15".as_bytes().to_vec(),
        }));
        let negative = Box::into_raw(Box::new(EchoString {
            bytes: "-7".as_bytes().to_vec(),
        }));
        let non_numeric = Box::into_raw(Box::new(EchoString {
            bytes: "abc".as_bytes().to_vec(),
        }));

        assert_eq!(echo_php_intval(EchoValue::null()), EchoValue::int(0));
        assert_eq!(echo_php_intval(EchoValue::bool(false)), EchoValue::int(0));
        assert_eq!(echo_php_intval(EchoValue::bool(true)), EchoValue::int(1));
        assert_eq!(echo_php_intval(EchoValue::int(42)), EchoValue::int(42));
        assert_eq!(echo_php_intval(EchoValue::string(empty)), EchoValue::int(0));
        assert_eq!(
            echo_php_intval(EchoValue::string(prefixed)),
            EchoValue::int(42)
        );
        assert_eq!(
            echo_php_intval(EchoValue::string(spaced)),
            EchoValue::int(15)
        );
        assert_eq!(
            echo_php_intval(EchoValue::string(negative)),
            EchoValue::int(-7)
        );
        assert_eq!(
            echo_php_intval(EchoValue::string(non_numeric)),
            EchoValue::int(0)
        );

        unsafe {
            drop(Box::from_raw(empty));
            drop(Box::from_raw(prefixed));
            drop(Box::from_raw(spaced));
            drop(Box::from_raw(negative));
            drop(Box::from_raw(non_numeric));
        }
    }

    #[test]
    fn floatval_preserves_php_scalar_float_coercion() {
        assert_float_value(echo_php_floatval(EchoValue::null()), 0.0);
        assert_float_value(echo_php_floatval(EchoValue::bool(true)), 1.0);
        assert_float_value(echo_php_floatval(EchoValue::int(42)), 42.0);

        let prefixed = Box::into_raw(Box::new(EchoString {
            bytes: b"122.34343The".to_vec(),
        }));
        let invalid = Box::into_raw(Box::new(EchoString {
            bytes: b"The122.34343".to_vec(),
        }));
        let offset = Box::into_raw(Box::new(EchoString {
            bytes: b"  -12.5px".to_vec(),
        }));
        let exponent = Box::into_raw(Box::new(EchoString {
            bytes: b"1e2x".to_vec(),
        }));

        assert_float_value(echo_php_floatval(EchoValue::string(prefixed)), 122.34343);
        assert_float_value(echo_php_floatval(EchoValue::string(invalid)), 0.0);
        assert_float_value(echo_php_floatval(EchoValue::string(offset)), -12.5);
        assert_float_value(echo_php_floatval(EchoValue::string(exponent)), 100.0);

        unsafe {
            drop(Box::from_raw(prefixed));
            drop(Box::from_raw(invalid));
            drop(Box::from_raw(offset));
            drop(Box::from_raw(exponent));
        }
    }

    #[test]
    fn float_scalar_math_builtins_preserve_php_scalar_behavior() {
        assert_float_value(echo_php_pi(), std::f64::consts::PI);
        assert_float_value(
            echo_php_fmod(EchoValue::float(5.7), EchoValue::float(1.3)),
            0.5,
        );
        assert_float_value(
            echo_php_fmod(EchoValue::float(-5.7), EchoValue::float(1.3)),
            -0.5,
        );
        assert!(
            f64::from_bits(echo_php_fmod(EchoValue::int(5), EchoValue::int(0)).payload).is_nan()
        );
    }

    #[test]
    fn string_unary_builtins_preserve_php_byte_behavior() {
        let reversed = Box::into_raw(Box::new(EchoString {
            bytes: "Echo ÄÖ 123!".as_bytes().to_vec(),
        }));
        let ucfirst = Box::into_raw(Box::new(EchoString {
            bytes: "echo".as_bytes().to_vec(),
        }));
        let lcfirst = Box::into_raw(Box::new(EchoString {
            bytes: "Echo".as_bytes().to_vec(),
        }));
        let non_ascii_first = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strrev(EchoValue::string(reversed)).string_bytes(),
            Some(vec![
                b'!', b'3', b'2', b'1', b' ', 0x96, 0xc3, 0x84, 0xc3, b' ', b'o', b'h', b'c', b'E'
            ])
        );
        assert_eq!(
            echo_php_ucfirst(EchoValue::string(ucfirst)).string_bytes(),
            Some("Echo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_lcfirst(EchoValue::string(lcfirst)).string_bytes(),
            Some("echo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_ucfirst(EchoValue::string(non_ascii_first)).string_bytes(),
            Some("Ächo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_lcfirst(EchoValue::string(non_ascii_first)).string_bytes(),
            Some("Ächo".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(reversed));
            drop(Box::from_raw(ucfirst));
            drop(Box::from_raw(lcfirst));
            drop(Box::from_raw(non_ascii_first));
        }
    }

    #[test]
    fn string_byte_builtins_preserve_php_byte_behavior() {
        let ascii = Box::into_raw(Box::new(EchoString {
            bytes: "A".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ä".as_bytes().to_vec(),
        }));
        let rot13 = Box::into_raw(Box::new(EchoString {
            bytes: "Echo PHP 4.3.0 ÄÖ!".as_bytes().to_vec(),
        }));

        assert_eq!(echo_php_ord(EchoValue::string(ascii)), EchoValue::int(65));
        assert_eq!(
            echo_php_ord(EchoValue::string(non_ascii)),
            EchoValue::int(195)
        );
        assert_eq!(
            echo_php_str_rot13(EchoValue::string(rot13)).string_bytes(),
            Some("Rpub CUC 4.3.0 ÄÖ!".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(ascii));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(rot13));
        }
    }

    #[test]
    fn base_string_conversion_builtins_preserve_php_byte_behavior() {
        assert_eq!(
            echo_php_chr(EchoValue::int(65)).string_bytes(),
            Some("A".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_chr(test_string_value(b"321")).string_bytes(),
            Some("A".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dechex(EchoValue::int(47)).string_bytes(),
            Some("2f".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dechex(EchoValue::int(-1)).string_bytes(),
            Some("ffffffffffffffff".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_decbin(EchoValue::int(26)).string_bytes(),
            Some("11010".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_decbin(EchoValue::int(-1)).string_bytes(),
            Some(
                "1111111111111111111111111111111111111111111111111111111111111111"
                    .as_bytes()
                    .to_vec()
            )
        );
        assert_eq!(
            echo_php_decoct(EchoValue::int(264)).string_bytes(),
            Some("410".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_decoct(EchoValue::int(-1)).string_bytes(),
            Some("1777777777777777777777".as_bytes().to_vec())
        );
    }

    #[test]
    fn bin2hex_preserves_php_byte_behavior() {
        assert_eq!(
            echo_php_bin2hex(test_string_value(b"Echo")).string_bytes(),
            Some("4563686f".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_bin2hex(test_string_value("Ä".as_bytes())).string_bytes(),
            Some("c384".as_bytes().to_vec())
        );
    }

    #[test]
    fn checksum_builtins_preserve_php_byte_behavior() {
        assert_eq!(
            echo_php_crc32(test_string_value(b"Echo\nPHP")),
            EchoValue::int(286159390)
        );
        assert_eq!(
            echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(false)).string_bytes(),
            Some("d4f2cb8de8248adb1e54f021bcd5e8c2".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(true)).string_bytes(),
            Some(vec![
                0xd4, 0xf2, 0xcb, 0x8d, 0xe8, 0x24, 0x8a, 0xdb, 0x1e, 0x54, 0xf0, 0x21, 0xbc, 0xd5,
                0xe8, 0xc2,
            ])
        );
        assert_eq!(
            echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(false)).string_bytes(),
            Some(
                "2ac003b31b44befef7f0c8b7e0154e3118689876"
                    .as_bytes()
                    .to_vec()
            )
        );
        assert_eq!(
            echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(true)).string_bytes(),
            Some(vec![
                0x2a, 0xc0, 0x03, 0xb3, 0x1b, 0x44, 0xbe, 0xfe, 0xf7, 0xf0, 0xc8, 0xb7, 0xe0, 0x15,
                0x4e, 0x31, 0x18, 0x68, 0x98, 0x76,
            ])
        );
    }

    #[test]
    fn base_to_decimal_builtins_preserve_php_unsigned_string_behavior() {
        assert_eq!(
            echo_php_bindec(test_string_value(b"1010")),
            EchoValue::int(10)
        );
        assert_eq!(
            echo_php_bindec(test_string_value(b"0b10xx11")),
            EchoValue::int(11)
        );
        assert_eq!(
            echo_php_hexdec(test_string_value(b"0xff")),
            EchoValue::int(255)
        );
        assert_eq!(
            echo_php_hexdec(test_string_value(b"ffzz10")),
            EchoValue::int(65296)
        );
        assert_eq!(
            echo_php_octdec(test_string_value(b"0789")),
            EchoValue::int(7)
        );
        assert_eq!(echo_php_bindec(EchoValue::int(10)), EchoValue::int(2));
        assert_eq!(echo_php_hexdec(EchoValue::float(10.7)), EchoValue::int(263));
        assert_eq!(echo_php_octdec(EchoValue::null()), EchoValue::int(0));
        assert_float_value(
            echo_php_hexdec(test_string_value(b"FFFFFFFFFFFFFFFF")),
            u64::MAX as f64,
        );
        assert_eq!(
            echo_php_base_convert(
                test_string_value(b"a37334"),
                EchoValue::int(16),
                EchoValue::int(2)
            )
            .string_bytes(),
            Some("101000110111001100110100".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base_convert(
                test_string_value(b"ffzz10"),
                EchoValue::int(16),
                EchoValue::int(10)
            )
            .string_bytes(),
            Some("65296".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base_convert(
                EchoValue::float(3.14),
                EchoValue::int(10),
                EchoValue::int(10)
            )
            .string_bytes(),
            Some("314".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base_convert(
                test_string_value(b"10"),
                EchoValue::int(1),
                EchoValue::int(10)
            ),
            EchoValue::error()
        );
    }

    #[test]
    fn escapeshellarg_preserves_php_unix_shell_argument_quoting() {
        assert_eq!(
            echo_php_escapeshellarg(test_string_value(b"Echo")).string_bytes(),
            Some("'Echo'".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_escapeshellarg(test_string_value(b"it's ready")).string_bytes(),
            Some("'it'\\''s ready'".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_escapeshellarg(test_string_value(b"")).string_bytes(),
            Some("''".as_bytes().to_vec())
        );
    }

    #[test]
    fn escapeshellcmd_preserves_php_unix_shell_meta_escaping() {
        fn string_value(bytes: &[u8]) -> *mut EchoString {
            Box::into_raw(Box::new(EchoString {
                bytes: bytes.to_vec(),
            }))
        }

        let semicolon = string_value(b"path; rm -rf /");
        let paired_double = string_value(b"echo \"ok\"");
        let unpaired_double = string_value(b"echo \"unterminated");
        let paired_single = string_value(b"echo 'ok'");
        let unpaired_single = string_value(b"echo 'unterminated");
        let newline = string_value(b"line\nbreak");
        let slash = string_value(b"a\\b");
        let dollar = string_value(b"a$b");

        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(semicolon)).string_bytes(),
            Some(b"path\\; rm -rf /".to_vec())
        );
        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(paired_double)).string_bytes(),
            Some(b"echo \"ok\"".to_vec())
        );
        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(unpaired_double)).string_bytes(),
            Some(b"echo \\\"unterminated".to_vec())
        );
        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(paired_single)).string_bytes(),
            Some(b"echo 'ok'".to_vec())
        );
        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(unpaired_single)).string_bytes(),
            Some(b"echo \\'unterminated".to_vec())
        );
        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(newline)).string_bytes(),
            Some(b"line\\\nbreak".to_vec())
        );
        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(slash)).string_bytes(),
            Some(b"a\\\\b".to_vec())
        );
        assert_eq!(
            echo_php_escapeshellcmd(EchoValue::string(dollar)).string_bytes(),
            Some(b"a\\$b".to_vec())
        );

        unsafe {
            drop(Box::from_raw(semicolon));
            drop(Box::from_raw(paired_double));
            drop(Box::from_raw(unpaired_double));
            drop(Box::from_raw(paired_single));
            drop(Box::from_raw(unpaired_single));
            drop(Box::from_raw(newline));
            drop(Box::from_raw(slash));
            drop(Box::from_raw(dollar));
        }
    }

    #[test]
    fn explode_preserves_php_array_count_and_limit_behavior() {
        fn string_value(bytes: &[u8]) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: bytes.to_vec(),
            })))
        }

        fn array_string_values(value: EchoValue) -> Vec<Vec<u8>> {
            let array = unsafe { (value.payload as *const EchoArray).as_ref() }.expect("array");
            array
                .values
                .iter()
                .map(|value| value.string_bytes().expect("string value"))
                .collect()
        }

        let all = echo_php_explode(
            string_value(b","),
            string_value(b"a,b,c"),
            EchoValue::int(i64::MAX),
        );
        let positive_limit = echo_php_explode(
            string_value(b","),
            string_value(b"a,b,c"),
            EchoValue::int(2),
        );
        let zero_limit = echo_php_explode(
            string_value(b","),
            string_value(b"a,b,c"),
            EchoValue::int(0),
        );
        let negative_limit = echo_php_explode(
            string_value(b","),
            string_value(b"a,b,c"),
            EchoValue::int(-1),
        );
        let missing_negative =
            echo_php_explode(string_value(b","), string_value(b"abc"), EchoValue::int(-1));
        let edge_empty = echo_php_explode(
            string_value(b","),
            string_value(b",a,"),
            EchoValue::int(i64::MAX),
        );

        assert_eq!(echo_php_count(all), EchoValue::int(3));
        assert_eq!(echo_php_array_is_list(all), EchoValue::bool(true));
        assert_eq!(all.string_bytes(), Some(b"Array".to_vec()));
        assert_eq!(
            array_string_values(all),
            vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]
        );
        assert_eq!(
            array_string_values(positive_limit),
            vec![b"a".to_vec(), b"b,c".to_vec()]
        );
        assert_eq!(array_string_values(zero_limit), vec![b"a,b,c".to_vec()]);
        assert_eq!(
            array_string_values(negative_limit),
            vec![b"a".to_vec(), b"b".to_vec()]
        );
        assert_eq!(array_string_values(missing_negative), Vec::<Vec<u8>>::new());
        assert_eq!(
            array_string_values(edge_empty),
            vec![Vec::new(), b"a".to_vec(), Vec::new()]
        );
        assert_eq!(
            echo_php_explode(
                string_value(b""),
                string_value(b"a,b"),
                EchoValue::int(i64::MAX)
            ),
            EchoValue::error()
        );
    }

    #[test]
    fn implode_joins_array_values_with_php_string_coercion() {
        let array = EchoValue::array(Box::into_raw(Box::new(EchoArray {
            keys: vec![
                EchoArrayKey::String(b"first".to_vec()),
                EchoArrayKey::Int(0),
                EchoArrayKey::String(b"third".to_vec()),
                EchoArrayKey::Int(1),
                EchoArrayKey::Int(2),
            ],
            values: vec![
                test_string_value(b"one"),
                EchoValue::int(2),
                EchoValue::bool(true),
                EchoValue::bool(false),
                EchoValue::null(),
            ],
        })));
        let empty = EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(Vec::new()))));

        assert_eq!(
            echo_php_implode(test_string_value(b"|"), array).string_bytes(),
            Some(b"one|2|1||".to_vec())
        );
        assert_eq!(
            echo_php_implode(test_string_value(b"hello"), empty).string_bytes(),
            Some(Vec::new())
        );
        assert_eq!(
            echo_php_implode(test_string_value(b","), test_string_value(b"not-array")),
            EchoValue::error()
        );
    }

    #[test]
    fn file_exists_reports_existing_files_and_directories() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let existing_file = manifest_dir.join("Cargo.toml");
        let existing_dir = manifest_dir.join("src");
        let missing_path = manifest_dir.join("definitely_missing_echo_file");
        let cargo_toml = Box::into_raw(Box::new(EchoString {
            bytes: existing_file.to_string_lossy().as_bytes().to_vec(),
        }));
        let src_dir = Box::into_raw(Box::new(EchoString {
            bytes: existing_dir.to_string_lossy().as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: missing_path.to_string_lossy().as_bytes().to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_file_exists(EchoValue::string(cargo_toml)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_file_exists(EchoValue::string(src_dir)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_file_exists(EchoValue::string(missing)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_file_exists(EchoValue::string(empty)),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(cargo_toml));
            drop(Box::from_raw(src_dir));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(empty));
        }
    }

    #[test]
    fn chdir_and_getcwd_preserve_php_working_directory_behavior() {
        let original = env::current_dir().expect("current dir");
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let missing = manifest_dir.join("definitely_missing_echo_directory");
        let original_bytes = original.to_string_lossy().as_bytes().to_vec();
        let missing_bytes = missing.to_string_lossy().as_bytes().to_vec();

        assert_eq!(
            echo_php_chdir(test_string_value(&original_bytes)),
            EchoValue::bool(true)
        );
        assert_eq!(echo_php_getcwd().string_bytes(), path_getcwd());
        assert_eq!(
            echo_php_chdir(test_string_value(&missing_bytes)),
            EchoValue::bool(false)
        );
    }

    #[test]
    fn is_dir_reports_only_existing_directories() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let existing_file = manifest_dir.join("Cargo.toml");
        let existing_dir = manifest_dir.join("src");
        let missing_path = manifest_dir.join("definitely_missing_echo_directory");
        let cargo_toml = Box::into_raw(Box::new(EchoString {
            bytes: existing_file.to_string_lossy().as_bytes().to_vec(),
        }));
        let src_dir = Box::into_raw(Box::new(EchoString {
            bytes: existing_dir.to_string_lossy().as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: missing_path.to_string_lossy().as_bytes().to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_is_dir(EchoValue::string(cargo_toml)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_dir(EchoValue::string(src_dir)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_dir(EchoValue::string(missing)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_dir(EchoValue::string(empty)),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(cargo_toml));
            drop(Box::from_raw(src_dir));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(empty));
        }
    }

    #[test]
    fn is_file_reports_only_existing_regular_files() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let existing_file = manifest_dir.join("Cargo.toml");
        let existing_dir = manifest_dir.join("src");
        let missing_path = manifest_dir.join("definitely_missing_echo_file");
        let cargo_toml = Box::into_raw(Box::new(EchoString {
            bytes: existing_file.to_string_lossy().as_bytes().to_vec(),
        }));
        let src_dir = Box::into_raw(Box::new(EchoString {
            bytes: existing_dir.to_string_lossy().as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: missing_path.to_string_lossy().as_bytes().to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_is_file(EchoValue::string(cargo_toml)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_file(EchoValue::string(src_dir)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_file(EchoValue::string(missing)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_file(EchoValue::string(empty)),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(cargo_toml));
            drop(Box::from_raw(src_dir));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(empty));
        }
    }

    #[cfg(unix)]
    #[test]
    fn is_link_reports_only_existing_symbolic_links() {
        let temp_dir =
            std::env::temp_dir().join(format!("echo-runtime-is-link-{}", std::process::id()));
        let target_path = temp_dir.join("target.txt");
        let link_path = temp_dir.join("linked-target.txt");
        std::fs::remove_dir_all(&temp_dir).ok();
        std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
        std::fs::write(&target_path, b"target").expect("write symlink target");
        std::os::unix::fs::symlink(&target_path, &link_path).expect("create symlink");

        let target = Box::into_raw(Box::new(EchoString {
            bytes: target_path.to_string_lossy().as_bytes().to_vec(),
        }));
        let link = Box::into_raw(Box::new(EchoString {
            bytes: link_path.to_string_lossy().as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: temp_dir
                .join("definitely_missing_echo_link")
                .to_string_lossy()
                .as_bytes()
                .to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_is_link(EchoValue::string(target)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_link(EchoValue::string(link)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_link(EchoValue::string(missing)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_link(EchoValue::string(empty)),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(target));
            drop(Box::from_raw(link));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(empty));
        }
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn filesystem_link_builtins_create_and_read_links() {
        let temp_dir =
            std::env::temp_dir().join(format!("echo-runtime-link-{}", std::process::id()));
        let target_path = temp_dir.join("target.txt");
        let symlink_path = temp_dir.join("target-link.txt");
        let hard_link_path = temp_dir.join("target-hard.txt");
        let missing_path = temp_dir.join("missing-link.txt");
        std::fs::remove_dir_all(&temp_dir).ok();
        std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
        std::fs::write(&target_path, b"target").expect("write link target");

        fn path_value(path: &Path) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: path.to_string_lossy().as_bytes().to_vec(),
            })))
        }

        let target = path_value(&target_path);
        let symlink = path_value(&symlink_path);
        let hard_link = path_value(&hard_link_path);
        let missing = path_value(&missing_path);

        assert_eq!(echo_php_symlink(target, symlink), EchoValue::bool(true));
        assert_eq!(echo_php_is_link(symlink), EchoValue::bool(true));
        assert_eq!(
            echo_php_readlink(symlink).string_bytes(),
            Some(target_path.to_string_lossy().as_bytes().to_vec())
        );
        assert_eq!(echo_php_link(target, hard_link), EchoValue::bool(true));
        assert_eq!(echo_php_is_link(hard_link), EchoValue::bool(false));
        assert_eq!(echo_php_file_exists(hard_link), EchoValue::bool(true));
        assert_eq!(echo_php_readlink(missing), EchoValue::bool(false));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn temporary_name_builtins_create_files_and_identifiers() {
        let temp_dir = std::env::temp_dir().join(format!(
            "echo-runtime-temporary-names-{}",
            std::process::id()
        ));
        std::fs::remove_dir_all(&temp_dir).ok();
        std::fs::create_dir_all(&temp_dir).expect("create temp test directory");

        fn string_value(bytes: &[u8]) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: bytes.to_vec(),
            })))
        }

        let sys_temp = echo_php_sys_get_temp_dir();
        let sys_temp_bytes = sys_temp.string_bytes().expect("temp dir string");
        assert!(path_is_dir(&sys_temp_bytes));

        let temp_file = echo_php_tempnam(
            string_value(temp_dir.to_string_lossy().as_bytes()),
            string_value(b"exo"),
        );
        let temp_file_bytes = temp_file.string_bytes().expect("tempnam string");
        assert!(path_is_file(&temp_file_bytes));
        assert!(
            path_buf_from_bytes(&temp_file_bytes)
                .and_then(|path| path.file_name().map(|name| name.to_owned()))
                .and_then(|name| name.into_string().ok())
                .is_some_and(|name| name.starts_with("exo"))
        );

        let plain = echo_php_uniqid(EchoValue::null(), EchoValue::bool(false))
            .string_bytes()
            .expect("uniqid string");
        let prefixed = echo_php_uniqid(string_value(b"job_"), EchoValue::bool(true))
            .string_bytes()
            .expect("prefixed uniqid string");
        assert_eq!(plain.len(), 13);
        assert_eq!(prefixed.len(), 27);
        assert!(prefixed.starts_with(b"job_"));

        std::fs::remove_file(path_buf_from_bytes(&temp_file_bytes).expect("tempnam path")).ok();
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn filesystem_metadata_builtins_report_paths_and_false_failures() {
        let temp_dir = std::env::temp_dir().join(format!(
            "echo-runtime-filesystem-metadata-{}",
            std::process::id()
        ));
        let file_path = temp_dir.join("sample.txt");
        let script_path = temp_dir.join("run.sh");
        let missing_path = temp_dir.join("missing.txt");
        std::fs::remove_dir_all(&temp_dir).ok();
        std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
        std::fs::write(&file_path, b"Echo file\n").expect("write sample file");
        std::fs::write(&script_path, b"#!/bin/sh\nexit 0\n").expect("write script file");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&script_path)
                .expect("stat script")
                .permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&script_path, permissions).expect("chmod script");
        }

        fn path_value(path: &Path) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: path.to_string_lossy().as_bytes().to_vec(),
            })))
        }

        let file = path_value(&file_path);
        let script = path_value(&script_path);
        let dir = path_value(&temp_dir);
        let missing = path_value(&missing_path);
        let parent_lookup = EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: temp_dir
                .join("..")
                .join(temp_dir.file_name().expect("temp dir name"))
                .join("sample.txt")
                .to_string_lossy()
                .as_bytes()
                .to_vec(),
        })));

        assert_eq!(echo_php_is_readable(file), EchoValue::bool(true));
        assert_eq!(echo_php_is_writable(file), EchoValue::bool(true));
        assert_eq!(echo_php_is_executable(file), EchoValue::bool(false));
        assert_eq!(echo_php_is_executable(script), EchoValue::bool(cfg!(unix)));
        assert_eq!(echo_php_is_readable(missing), EchoValue::bool(false));
        assert_eq!(echo_php_is_writable(missing), EchoValue::bool(false));
        assert_eq!(echo_php_filesize(file), EchoValue::int(10));
        assert_eq!(echo_php_filesize(missing), EchoValue::bool(false));
        assert_eq!(
            echo_php_filetype(file).string_bytes(),
            Some(b"file".to_vec())
        );
        assert_eq!(echo_php_filetype(dir).string_bytes(), Some(b"dir".to_vec()));
        assert!(echo_php_fileatime(file).is_int());
        assert!(echo_php_filectime(file).is_int());
        assert!(echo_php_filemtime(file).is_int());
        assert_eq!(echo_php_fileatime(missing), EchoValue::bool(false));
        assert_eq!(echo_php_filetype(missing), EchoValue::bool(false));
        #[cfg(unix)]
        {
            assert!(echo_php_fileinode(file).is_int());
            assert!(echo_php_fileowner(file).is_int());
            assert!(echo_php_filegroup(file).is_int());
            assert!(echo_php_fileperms(file).is_int());
        }
        assert_eq!(echo_php_realpath(missing), EchoValue::bool(false));
        assert_eq!(
            echo_php_realpath(parent_lookup).string_bytes(),
            path_realpath(file_path.to_string_lossy().as_bytes())
        );

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn filesystem_content_builtins_read_write_append_and_stream_output() {
        let temp_dir = std::env::temp_dir().join(format!(
            "echo-runtime-filesystem-content-{}",
            std::process::id()
        ));
        let file_path = temp_dir.join("report.txt");
        let missing_path = temp_dir.join("missing.txt");
        std::fs::remove_dir_all(&temp_dir).ok();
        std::fs::create_dir_all(&temp_dir).expect("create temp test directory");

        fn path_value(path: &Path) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: path.to_string_lossy().as_bytes().to_vec(),
            })))
        }

        fn string_value(bytes: &[u8]) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: bytes.to_vec(),
            })))
        }

        let file = path_value(&file_path);
        let missing = path_value(&missing_path);

        assert_eq!(
            echo_php_file_put_contents(
                file,
                string_value(b"alpha\nbeta\ngamma\n"),
                EchoValue::int(0),
                EchoValue::null()
            ),
            EchoValue::int(17)
        );
        assert_eq!(
            echo_php_file_put_contents(
                file,
                string_value(b"delta\n"),
                EchoValue::int(PHP_FILE_APPEND),
                EchoValue::null()
            ),
            EchoValue::int(6)
        );
        assert_eq!(
            echo_php_file_get_contents(
                file,
                EchoValue::bool(false),
                EchoValue::null(),
                EchoValue::int(6),
                EchoValue::int(4)
            )
            .string_bytes(),
            Some(b"beta".to_vec())
        );
        assert_eq!(
            echo_php_file_get_contents(
                file,
                EchoValue::bool(false),
                EchoValue::null(),
                EchoValue::int(-6),
                EchoValue::null()
            )
            .string_bytes(),
            Some(b"delta\n".to_vec())
        );
        assert_eq!(
            echo_php_file_get_contents(
                missing,
                EchoValue::bool(false),
                EchoValue::null(),
                EchoValue::int(0),
                EchoValue::null()
            ),
            EchoValue::bool(false)
        );

        let (bytes_read, stdout) = capture_stdout(false, || {
            echo_php_readfile(file, EchoValue::bool(false), EchoValue::null())
        });
        assert_eq!(bytes_read, EchoValue::int(23));
        assert_eq!(stdout, b"alpha\nbeta\ngamma\ndelta\n");

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn filesystem_mutation_builtins_create_move_and_remove_paths() {
        let temp_dir = std::env::temp_dir().join(format!(
            "echo-runtime-filesystem-mutation-{}",
            std::process::id()
        ));
        let nested_dir = temp_dir.join("cache").join("daily");
        let marker_path = nested_dir.join("marker.txt");
        let copied_path = nested_dir.join("marker-copy.txt");
        let renamed_path = nested_dir.join("marker-final.txt");
        let missing_path = nested_dir.join("missing.txt");
        std::fs::remove_dir_all(&temp_dir).ok();

        fn path_value(path: &Path) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: path.to_string_lossy().as_bytes().to_vec(),
            })))
        }

        let nested = path_value(&nested_dir);
        let marker = path_value(&marker_path);
        let copied = path_value(&copied_path);
        let renamed = path_value(&renamed_path);
        let missing = path_value(&missing_path);

        assert_eq!(
            echo_php_mkdir(
                nested,
                EchoValue::int(0o755),
                EchoValue::bool(true),
                EchoValue::null()
            ),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_mkdir(
                nested,
                EchoValue::int(0o755),
                EchoValue::bool(true),
                EchoValue::null()
            ),
            EchoValue::bool(false)
        );
        assert!(nested_dir.is_dir());

        assert_eq!(
            echo_php_touch(marker, EchoValue::int(1_700_000_000), EchoValue::null()),
            EchoValue::bool(true)
        );
        assert_eq!(echo_php_filemtime(marker), EchoValue::int(1_700_000_000));
        assert!(marker_path.is_file());

        assert_eq!(
            echo_php_copy(marker, copied, EchoValue::null()),
            EchoValue::bool(true)
        );
        assert!(copied_path.is_file());

        assert_eq!(
            echo_php_rename(copied, renamed, EchoValue::null()),
            EchoValue::bool(true)
        );
        assert!(!copied_path.exists());
        assert!(renamed_path.is_file());

        assert_eq!(
            echo_php_unlink(renamed, EchoValue::null()),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_unlink(missing, EchoValue::null()),
            EchoValue::bool(false)
        );
        assert!(!renamed_path.exists());

        assert_eq!(
            echo_php_unlink(marker, EchoValue::null()),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_rmdir(nested, EchoValue::null()),
            EchoValue::bool(true)
        );
        assert!(!nested_dir.exists());

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn base64_encode_preserves_php_byte_behavior() {
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let one_byte = Box::into_raw(Box::new(EchoString {
            bytes: "f".as_bytes().to_vec(),
        }));
        let two_bytes = Box::into_raw(Box::new(EchoString {
            bytes: "fo".as_bytes().to_vec(),
        }));
        let three_bytes = Box::into_raw(Box::new(EchoString {
            bytes: "foo".as_bytes().to_vec(),
        }));
        let text = Box::into_raw(Box::new(EchoString {
            bytes: "hello world".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_base64_encode(EchoValue::string(empty)).string_bytes(),
            Some(Vec::new())
        );
        assert_eq!(
            echo_php_base64_encode(EchoValue::string(one_byte)).string_bytes(),
            Some("Zg==".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_encode(EchoValue::string(two_bytes)).string_bytes(),
            Some("Zm8=".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_encode(EchoValue::string(three_bytes)).string_bytes(),
            Some("Zm9v".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_encode(EchoValue::string(text)).string_bytes(),
            Some("aGVsbG8gd29ybGQ=".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_encode(EchoValue::string(non_ascii)).string_bytes(),
            Some("w4RjaG8=".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_encode(EchoValue::int(123)).string_bytes(),
            Some("MTIz".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(empty));
            drop(Box::from_raw(one_byte));
            drop(Box::from_raw(two_bytes));
            drop(Box::from_raw(three_bytes));
            drop(Box::from_raw(text));
            drop(Box::from_raw(non_ascii));
        }
    }

    #[test]
    fn base64_decode_preserves_php_default_non_strict_byte_behavior() {
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let one_byte = Box::into_raw(Box::new(EchoString {
            bytes: "Zg==".as_bytes().to_vec(),
        }));
        let two_bytes = Box::into_raw(Box::new(EchoString {
            bytes: "Zm8=".as_bytes().to_vec(),
        }));
        let three_bytes = Box::into_raw(Box::new(EchoString {
            bytes: "Zm9v".as_bytes().to_vec(),
        }));
        let text = Box::into_raw(Box::new(EchoString {
            bytes: "aGVsbG8gd29ybGQ=".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "w4RjaG8=".as_bytes().to_vec(),
        }));
        let ignored = Box::into_raw(Box::new(EchoString {
            bytes: "Zm 9v".as_bytes().to_vec(),
        }));
        let invalid = Box::into_raw(Box::new(EchoString {
            bytes: "!!!!".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_base64_decode(EchoValue::string(empty)).string_bytes(),
            Some(Vec::new())
        );
        assert_eq!(
            echo_php_base64_decode(EchoValue::string(one_byte)).string_bytes(),
            Some("f".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_decode(EchoValue::string(two_bytes)).string_bytes(),
            Some("fo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_decode(EchoValue::string(three_bytes)).string_bytes(),
            Some("foo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_decode(EchoValue::string(text)).string_bytes(),
            Some("hello world".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_decode(EchoValue::string(non_ascii)).string_bytes(),
            Some("Ächo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_decode(EchoValue::string(ignored)).string_bytes(),
            Some("foo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_base64_decode(EchoValue::string(invalid)).string_bytes(),
            Some(Vec::new())
        );

        unsafe {
            drop(Box::from_raw(empty));
            drop(Box::from_raw(one_byte));
            drop(Box::from_raw(two_bytes));
            drop(Box::from_raw(three_bytes));
            drop(Box::from_raw(text));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(ignored));
            drop(Box::from_raw(invalid));
        }
    }

    #[test]
    fn url_encoding_builtins_preserve_php_byte_behavior() {
        fn string_value(bytes: &[u8]) -> EchoValue {
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: bytes.to_vec(),
            })))
        }

        assert_eq!(
            echo_php_rawurlencode(string_value(b"sales and marketing/Miami~")).string_bytes(),
            Some(b"sales%20and%20marketing%2FMiami~".to_vec())
        );
        assert_eq!(
            echo_php_urlencode(string_value(b"Data123!@-_ +~")).string_bytes(),
            Some(b"Data123%21%40-_+%2B%7E".to_vec())
        );
        assert_eq!(
            echo_php_rawurldecode(string_value(b"foo%20bar%40baz+plus%ZZ")).string_bytes(),
            Some(b"foo bar@baz+plus%ZZ".to_vec())
        );
        assert_eq!(
            echo_php_urldecode(string_value(b"green+and+red%2Bblue%ZZ")).string_bytes(),
            Some(b"green and red+blue%ZZ".to_vec())
        );
        assert_eq!(
            echo_php_rawurldecode(echo_php_rawurlencode(string_value(b"a/b c+~"))).string_bytes(),
            Some(b"a/b c+~".to_vec())
        );
        assert_eq!(
            echo_php_rawurlencode(string_value(&[0xc3, 0x84])).string_bytes(),
            Some(b"%C3%84".to_vec())
        );
    }

    #[test]
    fn angle_conversion_builtins_preserve_php_float_coercion() {
        assert_float_value(echo_php_deg2rad(EchoValue::int(180)), std::f64::consts::PI);
        assert_float_value(
            echo_php_rad2deg(EchoValue::float(std::f64::consts::PI)),
            180.0,
        );
        assert_float_value(
            echo_php_deg2rad(test_string_value(b"-90")),
            -std::f64::consts::FRAC_PI_2,
        );
        assert_float_value(echo_php_rad2deg(EchoValue::bool(true)), 57.29577951308232);
        assert_float_value(echo_php_deg2rad(EchoValue::null()), 0.0);
        assert_eq!(
            echo_php_deg2rad(test_string_value(b"not numeric")),
            EchoValue::error()
        );
    }

    #[test]
    fn trigonometric_builtins_preserve_php_float_coercion() {
        assert_float_value(
            echo_php_sin(EchoValue::float(std::f64::consts::FRAC_PI_6)),
            0.5,
        );
        assert_float_value(
            echo_php_cos(EchoValue::float(std::f64::consts::FRAC_PI_3)),
            0.5,
        );
        assert_float_value(
            echo_php_tan(EchoValue::float(std::f64::consts::FRAC_PI_4)),
            1.0,
        );
        assert_float_value(
            echo_php_asin(EchoValue::float(0.5)),
            std::f64::consts::FRAC_PI_6,
        );
        assert_float_value(
            echo_php_acos(EchoValue::float(0.5)),
            std::f64::consts::FRAC_PI_3,
        );
        assert_float_value(
            echo_php_atan(EchoValue::float(1.0)),
            std::f64::consts::FRAC_PI_4,
        );
        assert_float_value(
            echo_php_atan2(EchoValue::float(3.0), EchoValue::float(-3.0)),
            2.356194490192345,
        );
        assert_float_value(echo_php_sin(test_string_value(b"0.5")), 0.479425538604203);
        assert_float_value(echo_php_cos(EchoValue::bool(true)), 0.5403023058681398);
        assert!(f64::from_bits(echo_php_acos(EchoValue::int(2)).payload).is_nan());
    }

    #[test]
    fn hyperbolic_builtins_preserve_php_float_behavior() {
        assert_float_value(echo_php_sinh(EchoValue::int(0)), 0.0);
        assert_float_value(echo_php_sinh(EchoValue::int(1)), 1.1752011936438014);
        assert_float_value(echo_php_cosh(EchoValue::int(0)), 1.0);
        assert_float_value(echo_php_cosh(EchoValue::int(1)), 1.5430806348152437);
        assert_float_value(echo_php_tanh(EchoValue::int(1)), 0.7615941559557649);
        assert_float_value(echo_php_asinh(EchoValue::int(1)), 0.881373587019543);
        assert_float_value(echo_php_acosh(EchoValue::int(1)), 0.0);
        assert_float_value(echo_php_atanh(EchoValue::float(0.5)), 0.5493061443340548);
        assert_float_value(echo_php_cosh(test_string_value(b"2.5")), 6.132289479663687);
        assert!(f64::from_bits(echo_php_acosh(EchoValue::int(0)).payload).is_nan());
        assert!(f64::from_bits(echo_php_atanh(EchoValue::int(2)).payload).is_nan());
    }

    #[test]
    fn rounding_and_magnitude_builtins_preserve_php_float_behavior() {
        assert_float_value(echo_php_ceil(EchoValue::float(4.3)), 5.0);
        assert_float_value(echo_php_floor(EchoValue::float(9.999)), 9.0);
        assert_float_value(echo_php_floor(EchoValue::float(-3.14)), -4.0);
        assert_eq!(
            f64::from_bits(echo_php_ceil(EchoValue::float(-0.1)).payload).to_bits(),
            (-0.0f64).to_bits()
        );
        assert_float_value(echo_php_ceil(test_string_value(b"12.2")), 13.0);
        assert_float_value(echo_php_floor(EchoValue::bool(true)), 1.0);
        assert_float_value(echo_php_sqrt(EchoValue::int(9)), 3.0);
        assert_float_value(echo_php_sqrt(EchoValue::float(10.0)), 3.162277660168379);
        assert!(f64::from_bits(echo_php_sqrt(EchoValue::int(-1)).payload).is_nan());
        assert_float_value(echo_php_hypot(EchoValue::int(3), EchoValue::int(4)), 5.0);
        assert_float_value(
            echo_php_hypot(test_string_value(b"5"), test_string_value(b"12")),
            13.0,
        );
    }

    #[test]
    fn exponential_and_logarithm_builtins_preserve_php_float_behavior() {
        assert_float_value(echo_php_exp(EchoValue::int(0)), 1.0);
        assert_float_value(echo_php_expm1(EchoValue::int(0)), 0.0);
        assert_float_value(echo_php_log(EchoValue::int(8), EchoValue::int(2)), 3.0);
        assert_float_value(echo_php_log10(EchoValue::int(1000)), 3.0);
        assert_float_value(echo_php_log1p(EchoValue::int(0)), 0.0);
        assert_eq!(
            f64::from_bits(
                echo_php_log(EchoValue::int(0), EchoValue::float(std::f64::consts::E)).payload
            ),
            f64::NEG_INFINITY
        );
        assert!(
            f64::from_bits(
                echo_php_log(EchoValue::int(-1), EchoValue::float(std::f64::consts::E)).payload
            )
            .is_nan()
        );
        assert!(f64::from_bits(echo_php_log1p(EchoValue::int(-2)).payload).is_nan());
        assert_eq!(
            echo_php_log(EchoValue::int(8), EchoValue::int(0)),
            EchoValue::error()
        );
        assert_eq!(
            echo_php_pow(EchoValue::int(2), EchoValue::int(8)),
            EchoValue::int(256)
        );
        assert_float_value(echo_php_pow(EchoValue::int(10), EchoValue::int(-1)), 0.1);
        assert_float_value(
            echo_php_fdiv(EchoValue::int(125), EchoValue::int(100)),
            1.25,
        );
        assert_eq!(
            f64::from_bits(echo_php_fdiv(EchoValue::int(1), EchoValue::int(0)).payload),
            f64::INFINITY
        );
        assert_float_value(
            echo_php_fpow(EchoValue::float(1.05), EchoValue::int(2)),
            1.1025,
        );
        assert_eq!(
            f64::from_bits(echo_php_fpow(EchoValue::int(0), EchoValue::int(-2)).payload),
            f64::INFINITY
        );
        assert!(
            f64::from_bits(echo_php_fpow(EchoValue::int(-1), EchoValue::float(5.5)).payload)
                .is_nan()
        );
        assert!(
            f64::from_bits(echo_php_pow(EchoValue::int(-1), EchoValue::float(5.5)).payload)
                .is_nan()
        );
        assert_eq!(
            f64::from_bits(echo_php_pow(EchoValue::int(0), EchoValue::int(-1)).payload),
            f64::INFINITY
        );
    }

    #[test]
    fn basename_preserves_php_unix_path_string_behavior() {
        let path = Box::into_raw(Box::new(EchoString {
            bytes: "/etc/sudoers.d".as_bytes().to_vec(),
        }));
        let suffix = Box::into_raw(Box::new(EchoString {
            bytes: ".d".as_bytes().to_vec(),
        }));
        let trailing = Box::into_raw(Box::new(EchoString {
            bytes: "/etc/".as_bytes().to_vec(),
        }));
        let root = Box::into_raw(Box::new(EchoString {
            bytes: "/".as_bytes().to_vec(),
        }));
        let dot = Box::into_raw(Box::new(EchoString {
            bytes: ".".as_bytes().to_vec(),
        }));
        let empty_suffix = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let missing_suffix = Box::into_raw(Box::new(EchoString {
            bytes: ".txt".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_basename(EchoValue::string(path), EchoValue::string(suffix)).string_bytes(),
            Some("sudoers".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_basename(EchoValue::string(trailing), EchoValue::string(empty_suffix))
                .string_bytes(),
            Some("etc".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_basename(EchoValue::string(root), EchoValue::string(missing_suffix))
                .string_bytes(),
            Some(Vec::new())
        );
        assert_eq!(
            echo_php_basename(EchoValue::string(dot), EchoValue::string(empty_suffix))
                .string_bytes(),
            Some(".".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(path));
            drop(Box::from_raw(suffix));
            drop(Box::from_raw(trailing));
            drop(Box::from_raw(root));
            drop(Box::from_raw(dot));
            drop(Box::from_raw(empty_suffix));
            drop(Box::from_raw(missing_suffix));
        }
    }

    #[test]
    fn dirname_preserves_php_unix_path_string_behavior() {
        let file = Box::into_raw(Box::new(EchoString {
            bytes: "/etc/passwd".as_bytes().to_vec(),
        }));
        let trailing = Box::into_raw(Box::new(EchoString {
            bytes: "/etc/".as_bytes().to_vec(),
        }));
        let root = Box::into_raw(Box::new(EchoString {
            bytes: "/".as_bytes().to_vec(),
        }));
        let dot = Box::into_raw(Box::new(EchoString {
            bytes: ".".as_bytes().to_vec(),
        }));
        let relative = Box::into_raw(Box::new(EchoString {
            bytes: "foo/bar/baz".as_bytes().to_vec(),
        }));
        let repeated = Box::into_raw(Box::new(EchoString {
            bytes: "foo//bar".as_bytes().to_vec(),
        }));
        let nested = Box::into_raw(Box::new(EchoString {
            bytes: "/usr/local/lib".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_dirname(EchoValue::string(file), EchoValue::int(1)).string_bytes(),
            Some("/etc".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dirname(EchoValue::string(trailing), EchoValue::int(1)).string_bytes(),
            Some("/".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dirname(EchoValue::string(root), EchoValue::int(1)).string_bytes(),
            Some("/".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dirname(EchoValue::string(dot), EchoValue::int(1)).string_bytes(),
            Some(".".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dirname(EchoValue::string(relative), EchoValue::int(1)).string_bytes(),
            Some("foo/bar".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dirname(EchoValue::string(repeated), EchoValue::int(1)).string_bytes(),
            Some("foo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_dirname(EchoValue::string(nested), EchoValue::int(2)).string_bytes(),
            Some("/usr".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(file));
            drop(Box::from_raw(trailing));
            drop(Box::from_raw(root));
            drop(Box::from_raw(dot));
            drop(Box::from_raw(relative));
            drop(Box::from_raw(repeated));
            drop(Box::from_raw(nested));
        }
    }

    #[test]
    fn hex2bin_preserves_php_byte_behavior() {
        let hex = Box::into_raw(Box::new(EchoString {
            bytes: "c384".as_bytes().to_vec(),
        }));
        let upper_hex = Box::into_raw(Box::new(EchoString {
            bytes: "4563686F".as_bytes().to_vec(),
        }));
        let invalid_hex = Box::into_raw(Box::new(EchoString {
            bytes: "f".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_hex2bin(EchoValue::string(hex)).string_bytes(),
            Some("Ä".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_hex2bin(EchoValue::string(upper_hex)).string_bytes(),
            Some("Echo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_hex2bin(EchoValue::string(invalid_hex)),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(hex));
            drop(Box::from_raw(upper_hex));
            drop(Box::from_raw(invalid_hex));
        }
    }

    #[test]
    fn str_repeat_preserves_php_byte_behavior() {
        let repeated = Box::into_raw(Box::new(EchoString {
            bytes: "xo".as_bytes().to_vec(),
        }));
        let empty_repeat = Box::into_raw(Box::new(EchoString {
            bytes: "x".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_str_repeat(EchoValue::string(repeated), EchoValue::int(3)).string_bytes(),
            Some("xoxoxo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_str_repeat(EchoValue::string(empty_repeat), EchoValue::int(0)).string_bytes(),
            Some(Vec::new())
        );

        unsafe {
            drop(Box::from_raw(repeated));
            drop(Box::from_raw(empty_repeat));
        }
    }

    #[test]
    fn str_pad_preserves_php_byte_behavior() {
        let right = Box::into_raw(Box::new(EchoString {
            bytes: "ID".as_bytes().to_vec(),
        }));
        let left = Box::into_raw(Box::new(EchoString {
            bytes: "42".as_bytes().to_vec(),
        }));
        let both = Box::into_raw(Box::new(EchoString {
            bytes: "tag".as_bytes().to_vec(),
        }));
        let multi_left = Box::into_raw(Box::new(EchoString {
            bytes: "42".as_bytes().to_vec(),
        }));
        let multi_both = Box::into_raw(Box::new(EchoString {
            bytes: "go".as_bytes().to_vec(),
        }));
        let shorter = Box::into_raw(Box::new(EchoString {
            bytes: "already".as_bytes().to_vec(),
        }));
        let zero = Box::into_raw(Box::new(EchoString {
            bytes: "0".as_bytes().to_vec(),
        }));
        let dash = Box::into_raw(Box::new(EchoString {
            bytes: "-".as_bytes().to_vec(),
        }));
        let ab = Box::into_raw(Box::new(EchoString {
            bytes: "ab".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_str_pad(
                EchoValue::string(right),
                EchoValue::int(6),
                EchoValue::string(zero),
                EchoValue::int(1),
            )
            .string_bytes(),
            Some("ID0000".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_str_pad(
                EchoValue::string(left),
                EchoValue::int(5),
                EchoValue::string(zero),
                EchoValue::int(0),
            )
            .string_bytes(),
            Some("00042".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_str_pad(
                EchoValue::string(both),
                EchoValue::int(8),
                EchoValue::string(dash),
                EchoValue::int(2),
            )
            .string_bytes(),
            Some("--tag---".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_str_pad(
                EchoValue::string(multi_left),
                EchoValue::int(7),
                EchoValue::string(ab),
                EchoValue::int(0),
            )
            .string_bytes(),
            Some("ababa42".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_str_pad(
                EchoValue::string(multi_both),
                EchoValue::int(7),
                EchoValue::string(ab),
                EchoValue::int(2),
            )
            .string_bytes(),
            Some("abgoaba".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_str_pad(
                EchoValue::string(shorter),
                EchoValue::int(3),
                EchoValue::string(zero),
                EchoValue::int(0),
            )
            .string_bytes(),
            Some("already".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(right));
            drop(Box::from_raw(left));
            drop(Box::from_raw(both));
            drop(Box::from_raw(multi_left));
            drop(Box::from_raw(multi_both));
            drop(Box::from_raw(shorter));
            drop(Box::from_raw(zero));
            drop(Box::from_raw(dash));
            drop(Box::from_raw(ab));
        }
    }

    #[test]
    fn string_chunk_builtins_preserve_php_byte_behavior() {
        let split = Box::into_raw(Box::new(EchoString {
            bytes: "abcde".as_bytes().to_vec(),
        }));
        let empty_split = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let chunk = Box::into_raw(Box::new(EchoString {
            bytes: "abcde".as_bytes().to_vec(),
        }));
        let empty_chunk = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let pipe = Box::into_raw(Box::new(EchoString {
            bytes: "|".as_bytes().to_vec(),
        }));
        let crlf = Box::into_raw(Box::new(EchoString {
            bytes: "\r\n".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_implode(
                EchoValue::string(pipe),
                echo_php_str_split(EchoValue::string(split), EchoValue::int(2)),
            )
            .string_bytes(),
            Some("ab|cd|e".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_implode(
                EchoValue::string(pipe),
                echo_php_str_split(EchoValue::string(empty_split), EchoValue::int(1)),
            )
            .string_bytes(),
            Some(Vec::new())
        );
        assert_eq!(
            echo_php_chunk_split(
                EchoValue::string(chunk),
                EchoValue::int(2),
                EchoValue::string(pipe)
            )
            .string_bytes(),
            Some("ab|cd|e|".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_chunk_split(
                EchoValue::string(empty_chunk),
                EchoValue::int(2),
                EchoValue::string(pipe),
            )
            .string_bytes(),
            Some("|".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_chunk_split(
                EchoValue::int(12345),
                EchoValue::string(Box::into_raw(Box::new(EchoString {
                    bytes: "2".as_bytes().to_vec(),
                }))),
                EchoValue::string(crlf),
            )
            .string_bytes(),
            Some("12\r\n34\r\n5\r\n".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(split));
            drop(Box::from_raw(empty_split));
            drop(Box::from_raw(chunk));
            drop(Box::from_raw(empty_chunk));
            drop(Box::from_raw(pipe));
            drop(Box::from_raw(crlf));
        }
    }

    #[test]
    fn substr_preserves_php_byte_behavior() {
        let positive = Box::into_raw(Box::new(EchoString {
            bytes: "Echo PHP".as_bytes().to_vec(),
        }));
        let out_of_range = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric_offset = Box::into_raw(Box::new(EchoString {
            bytes: "1".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let negative = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_substr(EchoValue::string(positive), EchoValue::int(5)).string_bytes(),
            Some("PHP".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_substr(EchoValue::string(out_of_range), EchoValue::int(99)).string_bytes(),
            Some(Vec::new())
        );
        assert_eq!(
            echo_php_substr(EchoValue::string(negative), EchoValue::int(-2)).string_bytes(),
            Some("ef".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_substr(
                EchoValue::string(non_ascii),
                EchoValue::string(numeric_offset)
            )
            .string_bytes(),
            Some(vec![0x84, b'c', b'h', b'o'])
        );

        unsafe {
            drop(Box::from_raw(positive));
            drop(Box::from_raw(out_of_range));
            drop(Box::from_raw(numeric_offset));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(negative));
        }
    }

    #[test]
    fn strpos_preserves_php_byte_behavior() {
        let found_at_zero = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let found_later = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric_needle = Box::into_raw(Box::new(EchoString {
            bytes: "12345".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let needle_start = Box::into_raw(Box::new(EchoString {
            bytes: "ab".as_bytes().to_vec(),
        }));
        let needle_later = Box::into_raw(Box::new(EchoString {
            bytes: "cd".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "xy".as_bytes().to_vec(),
        }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "c".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strpos(
                EchoValue::string(found_at_zero),
                EchoValue::string(needle_start)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_strpos(
                EchoValue::string(found_later),
                EchoValue::string(needle_later)
            ),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_strpos(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_strpos(EchoValue::string(numeric_needle), EchoValue::int(34)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_strpos(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            ),
            EchoValue::int(2)
        );

        unsafe {
            drop(Box::from_raw(found_at_zero));
            drop(Box::from_raw(found_later));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(numeric_needle));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(needle_start));
            drop(Box::from_raw(needle_later));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_non_ascii));
        }
    }

    #[test]
    fn stripos_preserves_php_ascii_case_insensitive_byte_behavior() {
        let found_at_zero = Box::into_raw(Box::new(EchoString {
            bytes: "ABC".as_bytes().to_vec(),
        }));
        let found_later = Box::into_raw(Box::new(EchoString {
            bytes: "xxEcho".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let empty_needle = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric_needle = Box::into_raw(Box::new(EchoString {
            bytes: "12345".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let needle_start = Box::into_raw(Box::new(EchoString {
            bytes: "a".as_bytes().to_vec(),
        }));
        let needle_later = Box::into_raw(Box::new(EchoString {
            bytes: "ECHO".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "XY".as_bytes().to_vec(),
        }));
        let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "ä".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_stripos(
                EchoValue::string(found_at_zero),
                EchoValue::string(needle_start)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_stripos(
                EchoValue::string(found_later),
                EchoValue::string(needle_later)
            ),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_stripos(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_stripos(
                EchoValue::string(empty_needle),
                EchoValue::string(needle_empty)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_stripos(EchoValue::string(numeric_needle), EchoValue::int(34)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_stripos(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            ),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(found_at_zero));
            drop(Box::from_raw(found_later));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(empty_needle));
            drop(Box::from_raw(numeric_needle));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(needle_start));
            drop(Box::from_raw(needle_later));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_empty));
            drop(Box::from_raw(needle_non_ascii));
        }
    }

    #[test]
    fn strrpos_preserves_php_byte_behavior() {
        let repeated_start = Box::into_raw(Box::new(EchoString {
            bytes: "abcabc".as_bytes().to_vec(),
        }));
        let repeated_end = Box::into_raw(Box::new(EchoString {
            bytes: "abcabc".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let empty_needle = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric_needle = Box::into_raw(Box::new(EchoString {
            bytes: "1234545".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächocho".as_bytes().to_vec(),
        }));
        let needle_start = Box::into_raw(Box::new(EchoString {
            bytes: "ab".as_bytes().to_vec(),
        }));
        let needle_end = Box::into_raw(Box::new(EchoString {
            bytes: "bc".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "xy".as_bytes().to_vec(),
        }));
        let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "c".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strrpos(
                EchoValue::string(repeated_start),
                EchoValue::string(needle_start)
            ),
            EchoValue::int(3)
        );
        assert_eq!(
            echo_php_strrpos(
                EchoValue::string(repeated_end),
                EchoValue::string(needle_end)
            ),
            EchoValue::int(4)
        );
        assert_eq!(
            echo_php_strrpos(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_strrpos(
                EchoValue::string(empty_needle),
                EchoValue::string(needle_empty)
            ),
            EchoValue::int(6)
        );
        assert_eq!(
            echo_php_strrpos(EchoValue::string(numeric_needle), EchoValue::int(45)),
            EchoValue::int(5)
        );
        assert_eq!(
            echo_php_strrpos(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            ),
            EchoValue::int(5)
        );

        unsafe {
            drop(Box::from_raw(repeated_start));
            drop(Box::from_raw(repeated_end));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(empty_needle));
            drop(Box::from_raw(numeric_needle));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(needle_start));
            drop(Box::from_raw(needle_end));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_empty));
            drop(Box::from_raw(needle_non_ascii));
        }
    }

    #[test]
    fn strripos_preserves_php_ascii_case_insensitive_byte_behavior() {
        let repeated_start = Box::into_raw(Box::new(EchoString {
            bytes: "abABcd".as_bytes().to_vec(),
        }));
        let repeated_end = Box::into_raw(Box::new(EchoString {
            bytes: "abcABC".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let empty_needle = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric_needle = Box::into_raw(Box::new(EchoString {
            bytes: "1234545".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let needle_start = Box::into_raw(Box::new(EchoString {
            bytes: "aB".as_bytes().to_vec(),
        }));
        let needle_end = Box::into_raw(Box::new(EchoString {
            bytes: "BC".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "XY".as_bytes().to_vec(),
        }));
        let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "ä".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strripos(
                EchoValue::string(repeated_start),
                EchoValue::string(needle_start)
            ),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_strripos(
                EchoValue::string(repeated_end),
                EchoValue::string(needle_end)
            ),
            EchoValue::int(4)
        );
        assert_eq!(
            echo_php_strripos(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_strripos(
                EchoValue::string(empty_needle),
                EchoValue::string(needle_empty)
            ),
            EchoValue::int(6)
        );
        assert_eq!(
            echo_php_strripos(EchoValue::string(numeric_needle), EchoValue::int(45)),
            EchoValue::int(5)
        );
        assert_eq!(
            echo_php_strripos(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            ),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(repeated_start));
            drop(Box::from_raw(repeated_end));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(empty_needle));
            drop(Box::from_raw(numeric_needle));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(needle_start));
            drop(Box::from_raw(needle_end));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_empty));
            drop(Box::from_raw(needle_non_ascii));
        }
    }

    #[test]
    fn strstr_preserves_php_byte_behavior() {
        let email = Box::into_raw(Box::new(EchoString {
            bytes: "name@example.com".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let at_start = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "12345".as_bytes().to_vec(),
        }));
        let empty_needle = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let needle_at = Box::into_raw(Box::new(EchoString {
            bytes: "@".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "xy".as_bytes().to_vec(),
        }));
        let needle_start = Box::into_raw(Box::new(EchoString {
            bytes: "ab".as_bytes().to_vec(),
        }));
        let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "c".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strstr(EchoValue::string(email), EchoValue::string(needle_at)).string_bytes(),
            Some("@example.com".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strstr(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_strstr(EchoValue::string(at_start), EchoValue::string(needle_start))
                .string_bytes(),
            Some("abcdef".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strstr(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
            Some("345".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strstr(
                EchoValue::string(empty_needle),
                EchoValue::string(needle_empty)
            )
            .string_bytes(),
            Some("abcdef".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strstr(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            )
            .string_bytes(),
            Some("cho".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(email));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(at_start));
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(empty_needle));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(needle_at));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_start));
            drop(Box::from_raw(needle_empty));
            drop(Box::from_raw(needle_non_ascii));
        }
    }

    #[test]
    fn stristr_preserves_php_ascii_case_insensitive_byte_behavior() {
        let email = Box::into_raw(Box::new(EchoString {
            bytes: "USER@EXAMPLE.com".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let at_start = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "12345".as_bytes().to_vec(),
        }));
        let empty_needle = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let needle_email = Box::into_raw(Box::new(EchoString {
            bytes: "e".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "XY".as_bytes().to_vec(),
        }));
        let needle_start = Box::into_raw(Box::new(EchoString {
            bytes: "AB".as_bytes().to_vec(),
        }));
        let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "ä".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_stristr(EchoValue::string(email), EchoValue::string(needle_email))
                .string_bytes(),
            Some("ER@EXAMPLE.com".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_stristr(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_stristr(EchoValue::string(at_start), EchoValue::string(needle_start))
                .string_bytes(),
            Some("abcdef".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_stristr(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
            Some("345".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_stristr(
                EchoValue::string(empty_needle),
                EchoValue::string(needle_empty)
            )
            .string_bytes(),
            Some("abcdef".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_stristr(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            ),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(email));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(at_start));
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(empty_needle));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(needle_email));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_start));
            drop(Box::from_raw(needle_empty));
            drop(Box::from_raw(needle_non_ascii));
        }
    }

    #[test]
    fn strrchr_preserves_php_byte_behavior() {
        let email = Box::into_raw(Box::new(EchoString {
            bytes: "name@example.com".as_bytes().to_vec(),
        }));
        let repeated = Box::into_raw(Box::new(EchoString {
            bytes: "abcabc".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "1234545".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "ÄchoÄ".as_bytes().to_vec(),
        }));
        let empty_needle = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let needle_at = Box::into_raw(Box::new(EchoString {
            bytes: "@".as_bytes().to_vec(),
        }));
        let needle_repeated = Box::into_raw(Box::new(EchoString {
            bytes: "bc".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "xy".as_bytes().to_vec(),
        }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ä".as_bytes().to_vec(),
        }));
        let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_strrchr(EchoValue::string(email), EchoValue::string(needle_at)).string_bytes(),
            Some("@example.com".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strrchr(
                EchoValue::string(repeated),
                EchoValue::string(needle_repeated)
            )
            .string_bytes(),
            Some("bc".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strrchr(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_strrchr(EchoValue::string(numeric), EchoValue::int(45)).string_bytes(),
            Some("45".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strrchr(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            )
            .string_bytes(),
            Some("Ä".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strrchr(
                EchoValue::string(empty_needle),
                EchoValue::string(needle_empty)
            ),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(email));
            drop(Box::from_raw(repeated));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(empty_needle));
            drop(Box::from_raw(needle_at));
            drop(Box::from_raw(needle_repeated));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_non_ascii));
            drop(Box::from_raw(needle_empty));
        }
    }

    #[test]
    fn strpbrk_preserves_php_byte_mask_behavior() {
        let text = Box::into_raw(Box::new(EchoString {
            bytes: "This is a Simple text.".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "12345".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let empty_mask = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let mask_text = Box::into_raw(Box::new(EchoString {
            bytes: "mi".as_bytes().to_vec(),
        }));
        let mask_missing = Box::into_raw(Box::new(EchoString {
            bytes: "xy".as_bytes().to_vec(),
        }));
        let mask_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ä".as_bytes().to_vec(),
        }));
        let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_strpbrk(EchoValue::string(text), EchoValue::string(mask_text)).string_bytes(),
            Some("is is a Simple text.".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strpbrk(EchoValue::string(missing), EchoValue::string(mask_missing)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_strpbrk(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
            Some("345".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strpbrk(
                EchoValue::string(non_ascii),
                EchoValue::string(mask_non_ascii)
            )
            .string_bytes(),
            Some("Ächo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_strpbrk(EchoValue::string(empty_mask), EchoValue::string(mask_empty)),
            EchoValue::error()
        );

        unsafe {
            drop(Box::from_raw(text));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(empty_mask));
            drop(Box::from_raw(mask_text));
            drop(Box::from_raw(mask_missing));
            drop(Box::from_raw(mask_non_ascii));
            drop(Box::from_raw(mask_empty));
        }
    }

    #[test]
    fn strspn_preserves_php_byte_mask_behavior() {
        let digits = Box::into_raw(Box::new(EchoString {
            bytes: "42 is the answer".as_bytes().to_vec(),
        }));
        let prefix = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "12345".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString {
            bytes: "abc".as_bytes().to_vec(),
        }));
        let mask_digits = Box::into_raw(Box::new(EchoString {
            bytes: "0123456789".as_bytes().to_vec(),
        }));
        let mask_prefix = Box::into_raw(Box::new(EchoString {
            bytes: "abc".as_bytes().to_vec(),
        }));
        let mask_missing = Box::into_raw(Box::new(EchoString {
            bytes: "xyz".as_bytes().to_vec(),
        }));
        let mask_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Äc".as_bytes().to_vec(),
        }));
        let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_strspn(EchoValue::string(digits), EchoValue::string(mask_digits)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_strspn(EchoValue::string(prefix), EchoValue::string(mask_prefix)),
            EchoValue::int(3)
        );
        assert_eq!(
            echo_php_strspn(EchoValue::string(missing), EchoValue::string(mask_missing)),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_strspn(EchoValue::string(numeric), EchoValue::int(12)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_strspn(
                EchoValue::string(non_ascii),
                EchoValue::string(mask_non_ascii)
            ),
            EchoValue::int(3)
        );
        assert_eq!(
            echo_php_strspn(EchoValue::string(empty), EchoValue::string(mask_empty)),
            EchoValue::int(0)
        );

        unsafe {
            drop(Box::from_raw(digits));
            drop(Box::from_raw(prefix));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(empty));
            drop(Box::from_raw(mask_digits));
            drop(Box::from_raw(mask_prefix));
            drop(Box::from_raw(mask_missing));
            drop(Box::from_raw(mask_non_ascii));
            drop(Box::from_raw(mask_empty));
        }
    }

    #[test]
    fn strcspn_preserves_php_byte_mask_behavior() {
        let no_match = Box::into_raw(Box::new(EchoString {
            bytes: "abcd".as_bytes().to_vec(),
        }));
        let end_match = Box::into_raw(Box::new(EchoString {
            bytes: "abcd".as_bytes().to_vec(),
        }));
        let middle_match = Box::into_raw(Box::new(EchoString {
            bytes: "abcd".as_bytes().to_vec(),
        }));
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "12345".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ächo".as_bytes().to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString {
            bytes: "abc".as_bytes().to_vec(),
        }));
        let mask_no_match = Box::into_raw(Box::new(EchoString {
            bytes: "x".as_bytes().to_vec(),
        }));
        let mask_end_match = Box::into_raw(Box::new(EchoString {
            bytes: "d".as_bytes().to_vec(),
        }));
        let mask_middle_match = Box::into_raw(Box::new(EchoString {
            bytes: "bd".as_bytes().to_vec(),
        }));
        let mask_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "c".as_bytes().to_vec(),
        }));
        let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_strcspn(
                EchoValue::string(no_match),
                EchoValue::string(mask_no_match)
            ),
            EchoValue::int(4)
        );
        assert_eq!(
            echo_php_strcspn(
                EchoValue::string(end_match),
                EchoValue::string(mask_end_match)
            ),
            EchoValue::int(3)
        );
        assert_eq!(
            echo_php_strcspn(
                EchoValue::string(middle_match),
                EchoValue::string(mask_middle_match)
            ),
            EchoValue::int(1)
        );
        assert_eq!(
            echo_php_strcspn(EchoValue::string(numeric), EchoValue::int(34)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_strcspn(
                EchoValue::string(non_ascii),
                EchoValue::string(mask_non_ascii)
            ),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_strcspn(EchoValue::string(empty), EchoValue::string(mask_empty)),
            EchoValue::int(3)
        );

        unsafe {
            drop(Box::from_raw(no_match));
            drop(Box::from_raw(end_match));
            drop(Box::from_raw(middle_match));
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(empty));
            drop(Box::from_raw(mask_no_match));
            drop(Box::from_raw(mask_end_match));
            drop(Box::from_raw(mask_middle_match));
            drop(Box::from_raw(mask_non_ascii));
            drop(Box::from_raw(mask_empty));
        }
    }

    #[test]
    fn substr_count_preserves_php_non_overlapping_byte_behavior() {
        let words = Box::into_raw(Box::new(EchoString {
            bytes: "This is a test".as_bytes().to_vec(),
        }));
        let repeated = Box::into_raw(Box::new(EchoString {
            bytes: "aaaa".as_bytes().to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: "abcdef".as_bytes().to_vec(),
        }));
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "1234512345".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "ÄchoÄ".as_bytes().to_vec(),
        }));
        let empty_needle = Box::into_raw(Box::new(EchoString {
            bytes: "abc".as_bytes().to_vec(),
        }));
        let needle_words = Box::into_raw(Box::new(EchoString {
            bytes: "is".as_bytes().to_vec(),
        }));
        let needle_repeated = Box::into_raw(Box::new(EchoString {
            bytes: "aa".as_bytes().to_vec(),
        }));
        let needle_missing = Box::into_raw(Box::new(EchoString {
            bytes: "xy".as_bytes().to_vec(),
        }));
        let needle_non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ä".as_bytes().to_vec(),
        }));
        let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_substr_count(EchoValue::string(words), EchoValue::string(needle_words)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_substr_count(
                EchoValue::string(repeated),
                EchoValue::string(needle_repeated)
            ),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_substr_count(
                EchoValue::string(missing),
                EchoValue::string(needle_missing)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_substr_count(EchoValue::string(numeric), EchoValue::int(45)),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_substr_count(
                EchoValue::string(non_ascii),
                EchoValue::string(needle_non_ascii)
            ),
            EchoValue::int(2)
        );
        assert_eq!(
            echo_php_substr_count(
                EchoValue::string(empty_needle),
                EchoValue::string(needle_empty)
            ),
            EchoValue::error()
        );

        unsafe {
            drop(Box::from_raw(words));
            drop(Box::from_raw(repeated));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(empty_needle));
            drop(Box::from_raw(needle_words));
            drop(Box::from_raw(needle_repeated));
            drop(Box::from_raw(needle_missing));
            drop(Box::from_raw(needle_non_ascii));
            drop(Box::from_raw(needle_empty));
        }
    }

    #[test]
    fn strcmp_preserves_php_byte_sign_behavior() {
        let less_left = Box::into_raw(Box::new(EchoString {
            bytes: "a".as_bytes().to_vec(),
        }));
        let less_right = Box::into_raw(Box::new(EchoString {
            bytes: "b".as_bytes().to_vec(),
        }));
        let greater_left = Box::into_raw(Box::new(EchoString {
            bytes: "b".as_bytes().to_vec(),
        }));
        let greater_right = Box::into_raw(Box::new(EchoString {
            bytes: "a".as_bytes().to_vec(),
        }));
        let equal_left = Box::into_raw(Box::new(EchoString {
            bytes: "same".as_bytes().to_vec(),
        }));
        let equal_right = Box::into_raw(Box::new(EchoString {
            bytes: "same".as_bytes().to_vec(),
        }));
        let prefix_left = Box::into_raw(Box::new(EchoString {
            bytes: "abc".as_bytes().to_vec(),
        }));
        let prefix_right = Box::into_raw(Box::new(EchoString {
            bytes: "ab".as_bytes().to_vec(),
        }));
        let numeric_left = Box::into_raw(Box::new(EchoString {
            bytes: "123".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strcmp(EchoValue::string(less_left), EchoValue::string(less_right)),
            EchoValue::int(-1)
        );
        assert_eq!(
            echo_php_strcmp(
                EchoValue::string(greater_left),
                EchoValue::string(greater_right)
            ),
            EchoValue::int(1)
        );
        assert_eq!(
            echo_php_strcmp(
                EchoValue::string(equal_left),
                EchoValue::string(equal_right)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_strcmp(
                EchoValue::string(prefix_left),
                EchoValue::string(prefix_right)
            ),
            EchoValue::int(1)
        );
        assert_eq!(
            echo_php_strcmp(EchoValue::string(numeric_left), EchoValue::int(123)),
            EchoValue::int(0)
        );

        unsafe {
            drop(Box::from_raw(less_left));
            drop(Box::from_raw(less_right));
            drop(Box::from_raw(greater_left));
            drop(Box::from_raw(greater_right));
            drop(Box::from_raw(equal_left));
            drop(Box::from_raw(equal_right));
            drop(Box::from_raw(prefix_left));
            drop(Box::from_raw(prefix_right));
            drop(Box::from_raw(numeric_left));
        }
    }

    #[test]
    fn strcasecmp_preserves_php_ascii_case_insensitive_behavior() {
        let equal_left = Box::into_raw(Box::new(EchoString {
            bytes: "Echo".as_bytes().to_vec(),
        }));
        let equal_right = Box::into_raw(Box::new(EchoString {
            bytes: "echo".as_bytes().to_vec(),
        }));
        let less_left = Box::into_raw(Box::new(EchoString {
            bytes: "a".as_bytes().to_vec(),
        }));
        let less_right = Box::into_raw(Box::new(EchoString {
            bytes: "B".as_bytes().to_vec(),
        }));
        let greater_left = Box::into_raw(Box::new(EchoString {
            bytes: "B".as_bytes().to_vec(),
        }));
        let greater_right = Box::into_raw(Box::new(EchoString {
            bytes: "a".as_bytes().to_vec(),
        }));
        let prefix_left = Box::into_raw(Box::new(EchoString {
            bytes: "abc".as_bytes().to_vec(),
        }));
        let prefix_right = Box::into_raw(Box::new(EchoString {
            bytes: "AB".as_bytes().to_vec(),
        }));
        let numeric_left = Box::into_raw(Box::new(EchoString {
            bytes: "123".as_bytes().to_vec(),
        }));
        let non_ascii_left = Box::into_raw(Box::new(EchoString {
            bytes: "Ä".as_bytes().to_vec(),
        }));
        let non_ascii_right = Box::into_raw(Box::new(EchoString {
            bytes: "ä".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_strcasecmp(
                EchoValue::string(equal_left),
                EchoValue::string(equal_right)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_strcasecmp(EchoValue::string(less_left), EchoValue::string(less_right)),
            EchoValue::int(-1)
        );
        assert_eq!(
            echo_php_strcasecmp(
                EchoValue::string(greater_left),
                EchoValue::string(greater_right)
            ),
            EchoValue::int(1)
        );
        assert_eq!(
            echo_php_strcasecmp(
                EchoValue::string(prefix_left),
                EchoValue::string(prefix_right)
            ),
            EchoValue::int(1)
        );
        assert_eq!(
            echo_php_strcasecmp(EchoValue::string(numeric_left), EchoValue::int(123)),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_strcasecmp(
                EchoValue::string(non_ascii_left),
                EchoValue::string(non_ascii_right)
            ),
            EchoValue::int(-32)
        );

        unsafe {
            drop(Box::from_raw(equal_left));
            drop(Box::from_raw(equal_right));
            drop(Box::from_raw(less_left));
            drop(Box::from_raw(less_right));
            drop(Box::from_raw(greater_left));
            drop(Box::from_raw(greater_right));
            drop(Box::from_raw(prefix_left));
            drop(Box::from_raw(prefix_right));
            drop(Box::from_raw(numeric_left));
            drop(Box::from_raw(non_ascii_left));
            drop(Box::from_raw(non_ascii_right));
        }
    }

    #[test]
    fn strncmp_builtins_preserve_php_prefix_behavior() {
        let abc = Box::into_raw(Box::new(EchoString {
            bytes: b"abc".to_vec(),
        }));
        let abd = Box::into_raw(Box::new(EchoString {
            bytes: b"abd".to_vec(),
        }));
        let ab = Box::into_raw(Box::new(EchoString {
            bytes: b"ab".to_vec(),
        }));
        let upper_abd = Box::into_raw(Box::new(EchoString {
            bytes: b"ABD".to_vec(),
        }));

        assert_eq!(
            echo_php_strncmp(
                EchoValue::string(abc),
                EchoValue::string(abd),
                EchoValue::int(2)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_strncmp(
                EchoValue::string(abc),
                EchoValue::string(abd),
                EchoValue::int(3)
            ),
            EchoValue::int(-1)
        );
        assert_eq!(
            echo_php_strncmp(
                EchoValue::string(abc),
                EchoValue::string(ab),
                EchoValue::int(3)
            ),
            EchoValue::int(1)
        );
        assert_eq!(
            echo_php_strncasecmp(
                EchoValue::string(abc),
                EchoValue::string(upper_abd),
                EchoValue::int(2)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_strncasecmp(
                EchoValue::string(abc),
                EchoValue::string(upper_abd),
                EchoValue::int(3)
            ),
            EchoValue::int(-1)
        );

        unsafe {
            drop(Box::from_raw(abc));
            drop(Box::from_raw(abd));
            drop(Box::from_raw(ab));
            drop(Box::from_raw(upper_abd));
        }
    }

    #[test]
    fn substr_compare_preserves_php_offset_length_and_case_behavior() {
        let haystack = Box::into_raw(Box::new(EchoString {
            bytes: b"abcde".to_vec(),
        }));
        let needle_bc = Box::into_raw(Box::new(EchoString {
            bytes: b"bc".to_vec(),
        }));
        let needle_bcg = Box::into_raw(Box::new(EchoString {
            bytes: b"bcg".to_vec(),
        }));
        let needle_upper_bc = Box::into_raw(Box::new(EchoString {
            bytes: b"BC".to_vec(),
        }));
        let needle_cd = Box::into_raw(Box::new(EchoString {
            bytes: b"cd".to_vec(),
        }));

        assert_eq!(
            echo_php_substr_compare(
                EchoValue::string(haystack),
                EchoValue::string(needle_bc),
                EchoValue::int(1),
                EchoValue::int(2),
                EchoValue::bool(false)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_substr_compare(
                EchoValue::string(haystack),
                EchoValue::string(needle_bcg),
                EchoValue::int(1),
                EchoValue::int(2),
                EchoValue::bool(false)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_substr_compare(
                EchoValue::string(haystack),
                EchoValue::string(needle_upper_bc),
                EchoValue::int(1),
                EchoValue::int(2),
                EchoValue::bool(true)
            ),
            EchoValue::int(0)
        );
        assert_eq!(
            echo_php_substr_compare(
                EchoValue::string(haystack),
                EchoValue::string(needle_cd),
                EchoValue::int(1),
                EchoValue::int(2),
                EchoValue::bool(false)
            ),
            EchoValue::int(-1)
        );

        unsafe {
            drop(Box::from_raw(haystack));
            drop(Box::from_raw(needle_bc));
            drop(Box::from_raw(needle_bcg));
            drop(Box::from_raw(needle_upper_bc));
            drop(Box::from_raw(needle_cd));
        }
    }

    #[test]
    fn trim_builtins_strip_default_php_ascii_whitespace() {
        let trim = Box::into_raw(Box::new(EchoString {
            bytes: "\t Echo \n".as_bytes().to_vec(),
        }));
        let ltrim = Box::into_raw(Box::new(EchoString {
            bytes: "\t Echo \n".as_bytes().to_vec(),
        }));
        let rtrim = Box::into_raw(Box::new(EchoString {
            bytes: "\t Echo \n".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: " Ä ".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_trim(EchoValue::string(trim)).string_bytes(),
            Some("Echo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_ltrim(EchoValue::string(ltrim)).string_bytes(),
            Some("Echo \n".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_rtrim(EchoValue::string(rtrim)).string_bytes(),
            Some("\t Echo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_trim(EchoValue::string(non_ascii)).string_bytes(),
            Some("Ä".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(trim));
            drop(Box::from_raw(ltrim));
            drop(Box::from_raw(rtrim));
            drop(Box::from_raw(non_ascii));
        }
    }

    #[test]
    fn string_rewrite_builtins_preserve_php_byte_behavior() {
        let chop = Box::into_raw(Box::new(EchoString {
            bytes: b"invoice:1001\n".to_vec(),
        }));
        let quoted = Box::into_raw(Box::new(EchoString {
            bytes: b"a=b\nnext".to_vec(),
        }));
        let quoted_decode = Box::into_raw(Box::new(EchoString {
            bytes: b"a=3Db=0Anext".to_vec(),
        }));
        let nl2br = Box::into_raw(Box::new(EchoString {
            bytes: b"line1\nline2".to_vec(),
        }));
        let search = Box::into_raw(Box::new(EchoString {
            bytes: b"{{name}}".to_vec(),
        }));
        let replace = Box::into_raw(Box::new(EchoString {
            bytes: b"Ada".to_vec(),
        }));
        let subject = Box::into_raw(Box::new(EchoString {
            bytes: b"Hello {{name}}".to_vec(),
        }));
        let isearch = Box::into_raw(Box::new(EchoString {
            bytes: b"TOKEN".to_vec(),
        }));
        let ireplace = Box::into_raw(Box::new(EchoString {
            bytes: b"redacted".to_vec(),
        }));
        let isubject = Box::into_raw(Box::new(EchoString {
            bytes: b"token TOKEN".to_vec(),
        }));
        let tr_value = Box::into_raw(Box::new(EchoString {
            bytes: b"abc-123".to_vec(),
        }));
        let tr_from = Box::into_raw(Box::new(EchoString {
            bytes: b"abc123".to_vec(),
        }));
        let tr_to = Box::into_raw(Box::new(EchoString {
            bytes: b"xyz789".to_vec(),
        }));

        assert_eq!(
            echo_php_rtrim(EchoValue::string(chop)).string_bytes(),
            Some(b"invoice:1001".to_vec())
        );
        assert_eq!(
            echo_php_quoted_printable_encode(EchoValue::string(quoted)).string_bytes(),
            Some(b"a=3Db=0Anext".to_vec())
        );
        assert_eq!(
            echo_php_quoted_printable_decode(EchoValue::string(quoted_decode)).string_bytes(),
            Some(b"a=b\nnext".to_vec())
        );
        assert_eq!(
            echo_php_nl2br(EchoValue::string(nl2br), EchoValue::bool(false)).string_bytes(),
            Some(b"line1<br>\nline2".to_vec())
        );
        assert_eq!(
            echo_php_str_replace(
                EchoValue::string(search),
                EchoValue::string(replace),
                EchoValue::string(subject),
            )
            .string_bytes(),
            Some(b"Hello Ada".to_vec())
        );
        assert_eq!(
            echo_php_str_ireplace(
                EchoValue::string(isearch),
                EchoValue::string(ireplace),
                EchoValue::string(isubject),
            )
            .string_bytes(),
            Some(b"redacted redacted".to_vec())
        );
        assert_eq!(
            echo_php_strtr(
                EchoValue::string(tr_value),
                EchoValue::string(tr_from),
                EchoValue::string(tr_to),
            )
            .string_bytes(),
            Some(b"xyz-789".to_vec())
        );

        unsafe {
            drop(Box::from_raw(chop));
            drop(Box::from_raw(quoted));
            drop(Box::from_raw(quoted_decode));
            drop(Box::from_raw(nl2br));
            drop(Box::from_raw(search));
            drop(Box::from_raw(replace));
            drop(Box::from_raw(subject));
            drop(Box::from_raw(isearch));
            drop(Box::from_raw(ireplace));
            drop(Box::from_raw(isubject));
            drop(Box::from_raw(tr_value));
            drop(Box::from_raw(tr_from));
            drop(Box::from_raw(tr_to));
        }
    }

    #[test]
    fn abs_preserves_php_integer_absolute_value_behavior() {
        assert_eq!(echo_php_abs(EchoValue::int(42)), EchoValue::int(42));
        assert_eq!(echo_php_abs(EchoValue::int(-42)), EchoValue::int(42));
        assert_eq!(echo_php_abs(EchoValue::int(0)), EchoValue::int(0));
        assert_eq!(echo_php_abs(EchoValue::int(i64::MIN)), EchoValue::error());
        assert_eq!(echo_php_abs(EchoValue::bool(true)), EchoValue::error());
    }

    #[test]
    fn is_numeric_preserves_php_numeric_string_rules() {
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: b" 1337e0 ".to_vec(),
        }));
        let decimal = Box::into_raw(Box::new(EchoString {
            bytes: b"4.2".to_vec(),
        }));
        let hex_prefixed = Box::into_raw(Box::new(EchoString {
            bytes: b"0x539".to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_is_numeric(EchoValue::int(42)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_numeric(EchoValue::string(numeric)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_numeric(EchoValue::string(decimal)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_numeric(EchoValue::string(hex_prefixed)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_numeric(EchoValue::string(empty)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_numeric(EchoValue::bool(true)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_numeric(EchoValue::null()),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(decimal));
            drop(Box::from_raw(hex_prefixed));
            drop(Box::from_raw(empty));
        }
    }

    #[test]
    fn is_float_is_false_for_current_non_float_values() {
        let string = Box::into_raw(Box::new(EchoString {
            bytes: b"4.2".to_vec(),
        }));
        let array = Box::into_raw(Box::new(EchoArray::from_values(vec![EchoValue::int(1)])));

        assert_eq!(
            echo_php_is_float(EchoValue::int(42)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_float(EchoValue::string(string)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_float(EchoValue::array(array)),
            EchoValue::bool(false)
        );
        assert_eq!(echo_php_is_float(EchoValue::null()), EchoValue::bool(false));

        unsafe {
            drop(Box::from_raw(string));
            drop(Box::from_raw(array));
        }
    }

    #[test]
    fn is_finite_preserves_php_float_coercion_for_current_values() {
        let finite_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b" 4.2 ".to_vec(),
        }));
        let infinite_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b"1e9999".to_vec(),
        }));
        let non_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b"not numeric".to_vec(),
        }));
        let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

        assert_eq!(
            echo_php_is_finite(EchoValue::int(42)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_finite(EchoValue::bool(false)),
            EchoValue::bool(true)
        );
        assert_eq!(echo_php_is_finite(EchoValue::null()), EchoValue::bool(true));
        assert_eq!(
            echo_php_is_finite(EchoValue::string(finite_numeric)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_finite(EchoValue::string(infinite_numeric)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_finite(EchoValue::string(non_numeric)),
            EchoValue::error()
        );
        assert_eq!(
            echo_php_is_finite(EchoValue::array(array)),
            EchoValue::error()
        );

        unsafe {
            drop(Box::from_raw(finite_numeric));
            drop(Box::from_raw(infinite_numeric));
            drop(Box::from_raw(non_numeric));
            drop(Box::from_raw(array));
        }
    }

    #[test]
    fn is_infinite_preserves_php_float_coercion_for_current_values() {
        let finite_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b" 4.2 ".to_vec(),
        }));
        let infinite_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b"1e9999".to_vec(),
        }));
        let negative_infinite_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b"-1e9999".to_vec(),
        }));
        let non_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b"not numeric".to_vec(),
        }));
        let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

        assert_eq!(
            echo_php_is_infinite(EchoValue::int(42)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_infinite(EchoValue::bool(true)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_infinite(EchoValue::null()),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_infinite(EchoValue::string(finite_numeric)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_infinite(EchoValue::string(infinite_numeric)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_infinite(EchoValue::string(negative_infinite_numeric)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_infinite(EchoValue::string(non_numeric)),
            EchoValue::error()
        );
        assert_eq!(
            echo_php_is_infinite(EchoValue::array(array)),
            EchoValue::error()
        );

        unsafe {
            drop(Box::from_raw(finite_numeric));
            drop(Box::from_raw(infinite_numeric));
            drop(Box::from_raw(negative_infinite_numeric));
            drop(Box::from_raw(non_numeric));
            drop(Box::from_raw(array));
        }
    }

    #[test]
    fn is_nan_preserves_php_float_coercion_for_current_values() {
        let finite_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b" 4.2 ".to_vec(),
        }));
        let infinite_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b"1e9999".to_vec(),
        }));
        let non_numeric = Box::into_raw(Box::new(EchoString {
            bytes: b"not numeric".to_vec(),
        }));
        let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

        assert_eq!(echo_php_is_nan(EchoValue::int(42)), EchoValue::bool(false));
        assert_eq!(
            echo_php_is_nan(EchoValue::bool(true)),
            EchoValue::bool(false)
        );
        assert_eq!(echo_php_is_nan(EchoValue::null()), EchoValue::bool(false));
        assert_eq!(
            echo_php_is_nan(EchoValue::string(finite_numeric)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_nan(EchoValue::string(infinite_numeric)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_nan(EchoValue::string(non_numeric)),
            EchoValue::error()
        );
        assert_eq!(echo_php_is_nan(EchoValue::array(array)), EchoValue::error());

        unsafe {
            drop(Box::from_raw(finite_numeric));
            drop(Box::from_raw(infinite_numeric));
            drop(Box::from_raw(non_numeric));
            drop(Box::from_raw(array));
        }
    }

    #[test]
    fn is_object_reports_only_object_values() {
        let object = Box::into_raw(Box::new(EchoObject { fields: Vec::new() }));
        let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));
        let list = Box::into_raw(Box::new(EchoList { values: Vec::new() }));
        let string = Box::into_raw(Box::new(EchoString {
            bytes: b"value".to_vec(),
        }));

        assert_eq!(
            echo_php_is_object(EchoValue::object(object)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_object(EchoValue::array(array)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_object(EchoValue::list(list)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_object(EchoValue::string(string)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_object(EchoValue::int(42)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_object(EchoValue::bool(true)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_object(EchoValue::null()),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(object));
            drop(Box::from_raw(array));
            drop(Box::from_raw(list));
            drop(Box::from_raw(string));
        }
    }

    #[test]
    fn is_resource_reports_runtime_resource_handles() {
        let listener = Box::into_raw(Box::new(net::listen("127.0.0.1:0").expect("listen")));
        let object = Box::into_raw(Box::new(EchoObject { fields: Vec::new() }));
        let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

        assert_eq!(
            echo_php_is_resource(EchoValue::tcp_listener(listener)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_resource(EchoValue::object(object)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_resource(EchoValue::array(array)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_resource(EchoValue::int(42)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_resource(EchoValue::null()),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(listener));
            drop(Box::from_raw(object));
            drop(Box::from_raw(array));
        }
    }

    #[test]
    fn gettype_returns_php_type_names_for_current_values() {
        let string = Box::into_raw(Box::new(EchoString {
            bytes: b"abc".to_vec(),
        }));
        let list = Box::into_raw(Box::new(EchoList {
            values: vec![EchoValue::int(1)],
        }));

        assert_eq!(
            echo_php_gettype(EchoValue::null()).string_bytes(),
            Some(b"NULL".to_vec())
        );
        assert_eq!(
            echo_php_gettype(EchoValue::bool(false)).string_bytes(),
            Some(b"boolean".to_vec())
        );
        assert_eq!(
            echo_php_gettype(EchoValue::int(42)).string_bytes(),
            Some(b"integer".to_vec())
        );
        assert_eq!(
            echo_php_gettype(EchoValue::string(string)).string_bytes(),
            Some(b"string".to_vec())
        );
        assert_eq!(
            echo_php_gettype(EchoValue::list(list)).string_bytes(),
            Some(b"list".to_vec())
        );

        unsafe {
            drop(Box::from_raw(string));
            drop(Box::from_raw(list));
        }
    }

    #[test]
    fn lists_are_distinct_from_php_arrays() {
        let list = Box::into_raw(Box::new(EchoList {
            values: vec![EchoValue::int(1)],
        }));
        let value = EchoValue::list(list);

        assert_eq!(value.string_bytes(), Some(b"List".to_vec()));
        assert_eq!(echo_php_is_array(value), EchoValue::bool(false));
        assert_eq!(echo_php_is_countable(value), EchoValue::bool(true));
        assert_eq!(echo_php_is_iterable(value), EchoValue::bool(true));

        unsafe {
            drop(Box::from_raw(list));
        }
    }

    #[test]
    fn arrays_are_distinct_from_lists() {
        let array = Box::into_raw(Box::new(EchoArray::from_values(vec![
            EchoValue::int(1),
            EchoValue::int(2),
        ])));
        let value = EchoValue::array(array);

        assert_eq!(value.string_bytes(), Some(b"Array".to_vec()));
        assert_eq!(
            echo_std_reflect_type_of(value).string_bytes(),
            Some(b"array".to_vec())
        );
        assert_eq!(
            echo_php_gettype(value).string_bytes(),
            Some(b"array".to_vec())
        );
        assert_eq!(echo_php_count(value), EchoValue::int(2));
        assert_eq!(echo_php_is_array(value), EchoValue::bool(true));
        assert_eq!(echo_php_is_countable(value), EchoValue::bool(true));
        assert_eq!(echo_php_is_iterable(value), EchoValue::bool(true));

        unsafe {
            drop(Box::from_raw(array));
        }
    }

    #[test]
    fn function_exists_reports_supported_internal_builtins() {
        unsafe {
            register_reflection_for_test(
                "strlen",
                "string $string",
                "int",
                REFLECTION_SOURCE_PHP_BUILTIN,
            );
            register_reflection_for_test(
                "sizeof",
                "Countable|array $value",
                "int",
                REFLECTION_SOURCE_PHP_BUILTIN,
            );
        }

        let strlen = Box::into_raw(Box::new(EchoString {
            bytes: b"strlen".to_vec(),
        }));
        let uppercase = Box::into_raw(Box::new(EchoString {
            bytes: b"STRLEN".to_vec(),
        }));
        let alias = Box::into_raw(Box::new(EchoString {
            bytes: b"sizeof".to_vec(),
        }));
        let construct = Box::into_raw(Box::new(EchoString {
            bytes: b"echo".to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: b"definitely_missing_echo_builtin".to_vec(),
        }));

        assert_eq!(
            echo_php_function_exists(EchoValue::string(strlen)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_function_exists(EchoValue::string(uppercase)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_function_exists(EchoValue::string(alias)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_function_exists(EchoValue::string(construct)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_function_exists(EchoValue::string(missing)),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(strlen));
            drop(Box::from_raw(uppercase));
            drop(Box::from_raw(alias));
            drop(Box::from_raw(construct));
            drop(Box::from_raw(missing));
        }
    }

    #[test]
    fn is_callable_reports_registered_function_names() {
        unsafe {
            register_reflection_for_test(
                "fixture_callable_builtin",
                "",
                "",
                REFLECTION_SOURCE_PHP_BUILTIN,
            );
            register_reflection_for_test("fixture_callable_userland", "", "", 0);
        }

        let builtin = Box::into_raw(Box::new(EchoString {
            bytes: b"fixture_callable_builtin".to_vec(),
        }));
        let uppercase = Box::into_raw(Box::new(EchoString {
            bytes: b"FIXTURE_CALLABLE_BUILTIN".to_vec(),
        }));
        let userland = Box::into_raw(Box::new(EchoString {
            bytes: b"fixture_callable_userland".to_vec(),
        }));
        let missing = Box::into_raw(Box::new(EchoString {
            bytes: b"definitely_missing_callable".to_vec(),
        }));
        let non_utf8 = Box::into_raw(Box::new(EchoString { bytes: vec![0xff] }));
        let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

        assert_eq!(
            echo_php_is_callable(EchoValue::string(builtin)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_callable(EchoValue::string(uppercase)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_callable(EchoValue::string(userland)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_is_callable(EchoValue::string(missing)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_callable(EchoValue::string(non_utf8)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_callable(EchoValue::array(array)),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_is_callable(EchoValue::null()),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(builtin));
            drop(Box::from_raw(uppercase));
            drop(Box::from_raw(userland));
            drop(Box::from_raw(missing));
            drop(Box::from_raw(non_utf8));
            drop(Box::from_raw(array));
        }
    }

    unsafe fn register_reflection_for_test(
        name: &str,
        params: &str,
        return_type: &str,
        source_kind: i32,
    ) {
        unsafe {
            echo_reflection_register_function(
                name.as_ptr(),
                name.len(),
                params.as_ptr(),
                params.len(),
                return_type.as_ptr(),
                return_type.len(),
                source_kind,
            );
        }
    }

    #[test]
    fn reflect_type_of_reports_runtime_value_categories() {
        let string = Box::into_raw(Box::new(EchoString {
            bytes: b"text".to_vec(),
        }));
        let list = Box::into_raw(Box::new(EchoList { values: Vec::new() }));

        assert_eq!(
            echo_std_reflect_type_of(EchoValue::null()).string_bytes(),
            Some(b"null".to_vec())
        );
        assert_eq!(
            echo_std_reflect_type_of(EchoValue::bool(true)).string_bytes(),
            Some(b"bool".to_vec())
        );
        assert_eq!(
            echo_std_reflect_type_of(EchoValue::int(42)).string_bytes(),
            Some(b"int".to_vec())
        );
        assert_eq!(
            echo_std_reflect_type_of(EchoValue::string(string)).string_bytes(),
            Some(b"string".to_vec())
        );
        assert_eq!(
            echo_std_reflect_type_of(EchoValue::list(list)).string_bytes(),
            Some(b"list".to_vec())
        );

        unsafe {
            drop(Box::from_raw(string));
            drop(Box::from_raw(list));
        }
    }

    #[test]
    fn assert_intrinsics_report_success() {
        let left = Box::into_raw(Box::new(EchoString {
            bytes: b"same".to_vec(),
        }));
        let right = Box::into_raw(Box::new(EchoString {
            bytes: b"same".to_vec(),
        }));

        assert_eq!(
            echo_std_assert_ok(EchoValue::bool(true)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_std_assert_equals(EchoValue::int(42), EchoValue::int(42)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_std_assert_equals(EchoValue::string(left), EchoValue::string(right)),
            EchoValue::bool(true)
        );

        unsafe {
            drop(Box::from_raw(left));
            drop(Box::from_raw(right));
        }
    }

    #[test]
    fn string_predicate_builtins_are_binary_safe_and_case_sensitive() {
        let haystack = Box::into_raw(Box::new(EchoString {
            bytes: "Echo PHP".as_bytes().to_vec(),
        }));
        let matching = Box::into_raw(Box::new(EchoString {
            bytes: "PHP".as_bytes().to_vec(),
        }));
        let mismatched_case = Box::into_raw(Box::new(EchoString {
            bytes: "php".as_bytes().to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ä".as_bytes().to_vec(),
        }));
        let first_utf8_byte = Box::into_raw(Box::new(EchoString { bytes: vec![0xc3] }));

        assert_eq!(
            echo_php_str_contains(EchoValue::string(haystack), EchoValue::string(matching)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_str_contains(
                EchoValue::string(haystack),
                EchoValue::string(mismatched_case)
            ),
            EchoValue::bool(false)
        );
        assert_eq!(
            echo_php_str_contains(EchoValue::string(haystack), EchoValue::string(empty)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_str_starts_with(EchoValue::string(haystack), EchoValue::string(empty)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_str_ends_with(EchoValue::string(haystack), EchoValue::string(matching)),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_str_contains(
                EchoValue::string(non_ascii),
                EchoValue::string(first_utf8_byte)
            ),
            EchoValue::bool(true)
        );

        unsafe {
            drop(Box::from_raw(haystack));
            drop(Box::from_raw(matching));
            drop(Box::from_raw(mismatched_case));
            drop(Box::from_raw(empty));
            drop(Box::from_raw(non_ascii));
            drop(Box::from_raw(first_utf8_byte));
        }
    }

    #[test]
    fn string_escape_builtins_preserve_php_byte_behavior() {
        let quoted = Box::into_raw(Box::new(EchoString {
            bytes: vec![b'A', b'\'', b'"', b'\\', b'B'],
        }));
        let slashed_zero = Box::into_raw(Box::new(EchoString {
            bytes: b"\\0".to_vec(),
        }));
        let meta = Box::into_raw(Box::new(EchoString {
            bytes: b".\\+*?[^]($)".to_vec(),
        }));
        let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

        assert_eq!(
            echo_php_addslashes(EchoValue::string(quoted)).string_bytes(),
            Some(b"A\\'\\\"\\\\B".to_vec())
        );
        assert_eq!(
            echo_php_stripslashes(EchoValue::string(slashed_zero)).string_bytes(),
            Some(vec![0])
        );
        assert_eq!(
            echo_php_quotemeta(EchoValue::string(meta)).string_bytes(),
            Some(b"\\.\\\\\\+\\*\\?\\[\\^\\]\\(\\$\\)".to_vec())
        );
        assert_eq!(
            echo_php_quotemeta(EchoValue::string(empty)),
            EchoValue::bool(false)
        );

        unsafe {
            drop(Box::from_raw(quoted));
            drop(Box::from_raw(slashed_zero));
            drop(Box::from_raw(meta));
            drop(Box::from_raw(empty));
        }
    }

    #[test]
    fn environment_process_builtins_follow_php_shapes() {
        let key = format!("ECHO_RUNTIME_ENV_TEST_{}", std::process::id());
        let set_assignment = test_string_value(format!("{key}=staging").as_bytes());
        let empty_assignment = test_string_value(format!("{key}=").as_bytes());
        let unset_assignment = test_string_value(key.as_bytes());
        let key_value = test_string_value(key.as_bytes());

        assert_eq!(echo_php_putenv(set_assignment), EchoValue::bool(true));
        assert_eq!(
            echo_php_getenv(key_value, EchoValue::bool(false)).string_bytes(),
            Some(b"staging".to_vec())
        );

        assert_eq!(echo_php_putenv(empty_assignment), EchoValue::bool(true));
        assert_eq!(
            echo_php_getenv(key_value, EchoValue::bool(false)).string_bytes(),
            Some(Vec::new())
        );

        assert_eq!(echo_php_putenv(unset_assignment), EchoValue::bool(true));
        assert_eq!(
            echo_php_getenv(key_value, EchoValue::bool(false)),
            EchoValue::bool(false)
        );
        assert!(echo_php_getenv(EchoValue::null(), EchoValue::bool(false)).is_array());
        assert!(
            echo_php_gethostname().is_string() || echo_php_gethostname() == EchoValue::bool(false)
        );
        assert_eq!(echo_php_is_int(echo_php_getmypid()), EchoValue::bool(true));
    }
}
