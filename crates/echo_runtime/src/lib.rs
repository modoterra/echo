pub mod abi;
pub mod error;
pub mod io;
pub mod net;
pub mod poll;
pub mod process;
pub mod sched;
pub mod task;
pub mod task_group;
pub mod thread;
pub mod time;

use crc32fast::Hasher as Crc32Hasher;
use filetime::FileTime;
use md5_digest::{Digest as _, Md5};
use sha1::Sha1;
use std::cell::RefCell;
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{self as std_io, Write};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
pub struct OutputRuntime {
    stack: Vec<OutputBuffer>,
    implicit_flush: bool,
}

#[derive(Debug, Default)]
struct OutputBuffer {
    bytes: Vec<u8>,
    #[allow(dead_code)]
    callback: Option<EchoCallable>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoSymbol {
    name: String,
}

impl EchoSymbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoCallable {
    Function(EchoSymbol),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoError {
    InvalidCallable,
    UndefinedFunction(EchoSymbol),
}

#[derive(Debug)]
pub struct EchoString {
    bytes: Vec<u8>,
}

impl EchoString {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

#[derive(Debug)]
pub struct EchoList {
    values: Vec<EchoValue>,
}

impl EchoList {
    fn new() -> Self {
        Self { values: Vec::new() }
    }
}

#[derive(Debug)]
pub struct EchoArray {
    keys: Vec<EchoArrayKey>,
    values: Vec<EchoValue>,
}

impl EchoArray {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    fn from_values(values: Vec<EchoValue>) -> Self {
        let keys = (0..values.len())
            .map(|key| EchoArrayKey::Int(key as i64))
            .collect();
        Self { keys, values }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EchoArrayKey {
    Int(i64),
    String(Vec<u8>),
}

impl EchoArrayKey {
    fn from_value(value: EchoValue) -> Option<Self> {
        match value.kind {
            ECHO_VALUE_INT => Some(Self::Int(value.payload as i64)),
            ECHO_VALUE_FLOAT => Some(Self::Int(f64::from_bits(value.payload) as i64)),
            ECHO_VALUE_BOOL => Some(Self::Int(if value.payload == 0 { 0 } else { 1 })),
            ECHO_VALUE_NULL => Some(Self::String(Vec::new())),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(value.payload as *const EchoString).as_ref()?.bytes;
                match parse_php_array_integer_key(bytes) {
                    Some(key) => Some(Self::Int(key)),
                    None => Some(Self::String(bytes.clone())),
                }
            },
            _ => None,
        }
    }

    fn to_value(&self) -> EchoValue {
        match self {
            Self::Int(value) => EchoValue::int(*value),
            Self::String(bytes) => echo_runtime_string(bytes.clone()),
        }
    }
}

#[derive(Debug)]
pub struct EchoObject {
    fields: Vec<(String, EchoValue)>,
}

impl EchoObject {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeFunctionReflection {
    name: String,
    params_signature: String,
    return_type: String,
    source_kind: i32,
}

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

static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(1);
static NEXT_UNIQID_COUNTER: AtomicUsize = AtomicUsize::new(0);
static ASSERT_FAILURES: AtomicUsize = AtomicUsize::new(0);
static REQUIRED_ONCE_FILES: OnceLock<Mutex<HashSet<Vec<u8>>>> = OnceLock::new();

const REFLECTION_SOURCE_PHP_BUILTIN: i32 = 1;

const PHP_DEFAULT_TRIM_BYTES: &[u8] = b" \n\r\t\x0b\0";
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

    fn string_bytes(self) -> Option<Vec<u8>> {
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

    fn inspect_bytes(self) -> Option<Vec<u8>> {
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

    fn as_task_mut(self) -> Option<&'static mut task::EchoTask> {
        if self.kind != ECHO_VALUE_TASK || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut task::EchoTask).as_mut() }
    }

    fn as_task_group_mut(self) -> Option<&'static mut task_group::EchoTaskGroup> {
        if self.kind != ECHO_VALUE_TASK_GROUP || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut task_group::EchoTaskGroup).as_mut() }
    }

    fn as_process_mut(self) -> Option<&'static mut process::EchoProcess> {
        if self.kind != ECHO_VALUE_PROCESS || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut process::EchoProcess).as_mut() }
    }

    fn as_thread_mut(self) -> Option<&'static mut thread::EchoThread> {
        if self.kind != ECHO_VALUE_THREAD || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut thread::EchoThread).as_mut() }
    }

    fn as_tcp_listener_ref(self) -> Option<&'static net::EchoTcpListener> {
        if self.kind != ECHO_VALUE_TCP_LISTENER || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *const net::EchoTcpListener).as_ref() }
    }

    fn as_tcp_connection_mut(self) -> Option<&'static mut net::EchoTcpConnection> {
        if self.kind != ECHO_VALUE_TCP_CONNECTION || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut net::EchoTcpConnection).as_mut() }
    }
}

impl OutputRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, bytes: &[u8], stdout: &mut Vec<u8>) {
        match self.stack.last_mut() {
            Some(buffer) => buffer.bytes.extend_from_slice(bytes),
            None => stdout.extend_from_slice(bytes),
        }
    }

    pub fn ob_implicit_flush(&mut self, enabled: bool) {
        self.implicit_flush = enabled;
    }

    pub fn ob_start(&mut self) {
        self.ob_start_with_callback(None);
    }

    pub fn ob_start_with_callback(&mut self, callback: Option<EchoCallable>) {
        self.stack.push(OutputBuffer {
            bytes: Vec::new(),
            callback,
        });
    }

    pub fn ob_clean(&mut self) -> bool {
        // PHP `ob_clean()` discards active buffer contents without turning the buffer off.
        // Source: https://www.php.net/manual/en/function.ob-clean.php
        let Some(buffer) = self.stack.last_mut() else {
            return false;
        };

        buffer.bytes.clear();
        true
    }

    pub fn ob_flush(&mut self, stdout: &mut Vec<u8>) -> bool {
        if self.stack.is_empty() {
            return false;
        };

        // PHP flushes only the active buffer; nested buffers flush to their parent.
        // Sources: function.ob-flush.php and outcontrol.nesting-output-buffers.php
        let top = self.stack.len() - 1;
        let bytes = std::mem::take(&mut self.stack[top].bytes);

        match top
            .checked_sub(1)
            .and_then(|parent| self.stack.get_mut(parent))
        {
            Some(parent) => parent.bytes.extend_from_slice(&bytes),
            None => stdout.extend_from_slice(&bytes),
        }

        true
    }

    pub fn ob_end_flush(&mut self, stdout: &mut Vec<u8>) -> bool {
        // PHP `ob_end_flush()` flushes contents and turns off the active buffer.
        // Source: https://www.php.net/manual/en/function.ob-end-flush.php
        let Some(buffer) = self.stack.pop() else {
            return false;
        };

        self.write(&buffer.bytes, stdout);
        true
    }

    pub fn ob_end_clean(&mut self) -> bool {
        // PHP `ob_end_clean()` discards contents and turns off the active buffer.
        // Source: https://www.php.net/manual/en/function.ob-end-clean.php
        self.take_active_buffer().is_some()
    }

    pub fn ob_get_clean(&mut self) -> Option<EchoString> {
        // PHP `ob_get_clean()` returns the active buffer contents and turns that buffer off.
        // Source: https://www.php.net/manual/en/function.ob-get-clean.php
        self.take_active_buffer()
            .map(|buffer| EchoString { bytes: buffer })
    }

    pub fn ob_get_flush(&mut self, stdout: &mut Vec<u8>) -> Option<EchoString> {
        // PHP `ob_get_flush()` returns the active buffer contents, flushes them, and turns it off.
        // Source: https://www.php.net/manual/en/function.ob-get-flush.php
        let buffer = self.take_active_buffer()?;
        self.write(&buffer, stdout);
        Some(EchoString { bytes: buffer })
    }

    pub fn ob_get_contents(&self) -> Option<EchoString> {
        // PHP `ob_get_contents()` returns a new string with the active buffer contents.
        // Source: https://www.php.net/manual/en/function.ob-get-contents.php
        self.stack.last().map(|buffer| EchoString {
            bytes: buffer.bytes.clone(),
        })
    }

    pub fn ob_get_length(&self) -> Option<usize> {
        // PHP `ob_get_length()` returns the active buffer length in bytes.
        // Source: https://www.php.net/manual/en/function.ob-get-length.php
        self.stack.last().map(|buffer| buffer.bytes.len())
    }

    pub fn shutdown(&mut self, stdout: &mut Vec<u8>) {
        // PHP shutdown flushes and turns off still-open buffers in reverse start order.
        // Source: https://www.php.net/manual/en/outcontrol.user-level-output-buffers.php
        while self.ob_end_flush(stdout) {}
    }

    pub fn level(&self) -> usize {
        self.stack.len()
    }

    fn take_active_buffer(&mut self) -> Option<Vec<u8>> {
        self.stack.pop().map(|buffer| buffer.bytes)
    }
}

pub fn echo_is_callable(value: EchoValue) -> bool {
    echo_normalize_callable(value).is_ok_and(|callback| callback.is_some())
}

pub fn echo_normalize_callable(value: EchoValue) -> Result<Option<EchoCallable>, EchoError> {
    if value.is_null() {
        return Ok(None);
    }

    if value.is_string() {
        let string = unsafe { (value.payload as *const EchoString).as_ref() }
            .ok_or(EchoError::InvalidCallable)?;
        let name = std::str::from_utf8(&string.bytes).map_err(|_| EchoError::InvalidCallable)?;

        return Ok(Some(EchoCallable::Function(EchoSymbol::new(name))));
    }

    Err(EchoError::InvalidCallable)
}

pub fn echo_call(callable: &EchoCallable, _args: &[EchoValue]) -> Result<EchoValue, EchoError> {
    match callable {
        EchoCallable::Function(symbol) if symbol.as_str() == "ob_start" => {
            OUTPUT.with(|runtime| runtime.borrow_mut().ob_start());
            Ok(EchoValue::null())
        }
        EchoCallable::Function(symbol) => Err(EchoError::UndefinedFunction(symbol.clone())),
    }
}

thread_local! {
    static OUTPUT: RefCell<OutputRuntime> = RefCell::new(OutputRuntime::new());
    static EXECUTION: RefCell<RuntimeExecution> = RefCell::new(RuntimeExecution::default());
}

#[derive(Debug, Default)]
struct RuntimeExecution {
    stdout: Option<Vec<u8>>,
    repl_inspect: bool,
}

pub fn reset_execution_state() {
    OUTPUT.with(|runtime| {
        *runtime.borrow_mut() = OutputRuntime::new();
    });
    ASSERT_FAILURES.store(0, Ordering::Relaxed);
}

pub fn capture_stdout<T>(repl_inspect: bool, f: impl FnOnce() -> T) -> (T, Vec<u8>) {
    reset_execution_state();
    EXECUTION.with(|execution| {
        *execution.borrow_mut() = RuntimeExecution {
            stdout: Some(Vec::new()),
            repl_inspect,
        };
    });

    let result = f();
    let stdout = EXECUTION.with(|execution| {
        let mut execution = execution.borrow_mut();
        execution.repl_inspect = false;
        execution.stdout.take().unwrap_or_default()
    });

    (result, stdout)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_write(ptr: *const u8, len: usize) {
    if ptr.is_null() && len != 0 {
        return;
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        runtime.borrow_mut().write(bytes, &mut stdout);
        write_stdout(&stdout);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_write_i64(value: i64) {
    let bytes = value.to_string();
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        runtime.borrow_mut().write(bytes.as_bytes(), &mut stdout);
        write_stdout(&stdout);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_write_i64_or_false(value: i64) {
    // PHP echoes `false` as an empty string. Echo uses -1 as the current sentinel
    // for int|false runtime calls where supported integer results cannot be negative.
    if value >= 0 {
        echo_write_i64(value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_write_string(value: *const EchoString) {
    if value.is_null() {
        return;
    }

    let bytes = unsafe { &(*value).bytes };
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        runtime.borrow_mut().write(bytes, &mut stdout);
        write_stdout(&stdout);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_write_value(value: EchoValue) {
    if repl_inspect_enabled() {
        if let Some(bytes) = value.inspect_bytes() {
            unsafe { echo_write(bytes.as_ptr(), bytes.len()) };
        }
        return;
    }

    match value.kind {
        ECHO_VALUE_NULL | ECHO_VALUE_ERROR => {}
        ECHO_VALUE_BOOL => {
            if value.payload != 0 {
                unsafe { echo_write(c"1".as_ptr().cast(), 1) };
            }
        }
        ECHO_VALUE_INT => echo_write_i64(value.payload as i64),
        ECHO_VALUE_FLOAT => {
            let bytes = format_php_float(f64::from_bits(value.payload)).into_bytes();
            unsafe { echo_write(bytes.as_ptr(), bytes.len()) };
        }
        ECHO_VALUE_STRING => unsafe { echo_write_string(value.payload as *const EchoString) },
        ECHO_VALUE_ARRAY => unsafe { echo_write(c"Array".as_ptr().cast(), 5) },
        ECHO_VALUE_LIST => unsafe { echo_write(c"List".as_ptr().cast(), 4) },
        _ => {}
    }
}

fn repl_inspect_enabled() -> bool {
    EXECUTION.with(|execution| execution.borrow().repl_inspect)
        || std::env::var_os("ECHO_REPL_INSPECT").is_some()
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
pub extern "C" fn echo_php_flush() {
    let _ = std_io::stdout().flush();
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_implicit_flush(value: EchoValue) {
    OUTPUT.with(|runtime| {
        runtime
            .borrow_mut()
            .ob_implicit_flush(value.bool_value().unwrap_or(false));
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_string(ptr: *const u8, len: usize) -> EchoValue {
    if ptr.is_null() && len != 0 {
        return EchoValue::error();
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_defer(callback: Option<task::EchoTaskCallback>) -> EchoValue {
    let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
    EchoValue::task(Box::into_raw(Box::new(task::EchoTask::deferred(
        task::TaskId(id),
        callback,
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_run(task_value: EchoValue) -> EchoValue {
    let Some(task) = task_value.as_task_mut() else {
        return EchoValue::error();
    };

    match sched::with_thread_event_loop(|event_loop| {
        event_loop
            .schedule_task(task)
            .map_err(|_| std_io::Error::other("failed to schedule Echo task"))
    }) {
        Ok(()) => task_value,
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_join(task_value: EchoValue) -> EchoValue {
    let Some(task) = task_value.as_task_mut() else {
        return EchoValue::error();
    };

    match task.result() {
        Ok(value) => return value,
        Err(task::TaskResultError::Failed) => return EchoValue::error(),
        Err(task::TaskResultError::NotFinished) => {}
    }

    sched::with_thread_event_loop(|event_loop| {
        event_loop
            .join_task(task)
            .map_err(|_| std_io::Error::other("failed to join Echo task"))
    })
    .unwrap_or_else(|_| EchoValue::error())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_group_new() -> EchoValue {
    EchoValue::task_group(Box::into_raw(Box::new(task_group::EchoTaskGroup::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_group_add(group_value: EchoValue, task_value: EchoValue) -> EchoValue {
    let Some(group) = group_value.as_task_group_mut() else {
        return EchoValue::error();
    };
    let Some(task) = task_value.as_task_mut() else {
        return EchoValue::error();
    };

    group.add(task as *mut task::EchoTask);
    group_value
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_group_run_and_join(group_value: EchoValue) -> EchoValue {
    let Some(group) = group_value.as_task_group_mut() else {
        return EchoValue::error();
    };

    let schedule_result = sched::with_thread_event_loop(|event_loop| {
        for task in &group.tasks {
            let Some(task) = (unsafe { task.as_ref() }) else {
                return Err(std_io::Error::other("invalid Echo task in group"));
            };
            event_loop
                .schedule_task(task)
                .map_err(|_| std_io::Error::other("failed to schedule Echo task group task"))?;
        }
        Ok(())
    });
    if schedule_result.is_err() {
        return EchoValue::error();
    }

    let mut results = EchoList::new();
    for task in &group.tasks {
        let Some(task) = (unsafe { task.as_ref() }) else {
            return EchoValue::error();
        };
        let result = match task.result() {
            Ok(value) => value,
            Err(task::TaskResultError::Failed) => return EchoValue::error(),
            Err(task::TaskResultError::NotFinished) => {
                match sched::with_thread_event_loop(|event_loop| {
                    event_loop
                        .join_task(task)
                        .map_err(|_| std_io::Error::other("failed to join Echo task group task"))
                }) {
                    Ok(value) => value,
                    Err(_) => return EchoValue::error(),
                }
            }
        };
        results.values.push(result);
    }

    EchoValue::list(Box::into_raw(Box::new(results)))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_thread_fork(callback: Option<task::EchoTaskCallback>) -> EchoValue {
    let Some(callback) = callback else {
        return EchoValue::error();
    };
    let id = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
    let thread = thread::EchoThread::fork(task::ThreadId(id), callback);

    EchoValue::thread(Box::into_raw(Box::new(thread)))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_thread_fork_task(task_value: EchoValue) -> EchoValue {
    let Some(task) = task_value.as_task_mut() else {
        return EchoValue::error();
    };
    let Some(callback) = task.callback() else {
        return EchoValue::error();
    };
    let id = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
    let thread = thread::EchoThread::fork(task::ThreadId(id), callback);

    EchoValue::thread(Box::into_raw(Box::new(thread)))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_thread_join(thread_value: EchoValue) -> EchoValue {
    let Some(thread) = thread_value.as_thread_mut() else {
        return EchoValue::error();
    };

    thread.join()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_process_spawn(command: EchoValue) -> EchoValue {
    let Some(command) = command.string_bytes() else {
        return EchoValue::error();
    };
    let id = NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed);

    match process::EchoProcess::spawn(task::ProcessId(id), command) {
        Ok(process) => EchoValue::process(Box::into_raw(Box::new(process))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_process_join(process_value: EchoValue) -> EchoValue {
    let Some(process) = process_value.as_process_mut() else {
        return EchoValue::error();
    };

    process.join()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_join(handle: EchoValue) -> EchoValue {
    match handle.kind {
        ECHO_VALUE_TASK => echo_task_join(handle),
        ECHO_VALUE_THREAD => echo_thread_join(handle),
        ECHO_VALUE_PROCESS => echo_process_join(handle),
        _ => {
            eprintln!("error: join target is not a task, thread, or process handle");
            EchoValue::error()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_time_sleep(millis: i64) {
    if millis <= 0 {
        return;
    }

    std::thread::sleep(Duration::from_millis(millis as u64));
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_sleep_current(
    millis: i64,
    continuation: Option<task::EchoTaskCallback>,
) -> EchoValue {
    if sched::sleep_current_task(millis, continuation) {
        EchoValue::pending()
    } else {
        echo_time_sleep(millis);
        EchoValue::null()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_listen(address: EchoValue) -> EchoValue {
    let Some(bytes) = address.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(address) = String::from_utf8(bytes) else {
        return EchoValue::error();
    };

    match net::listen(address) {
        Ok(listener) => EchoValue::tcp_listener(Box::into_raw(Box::new(listener))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_connect(address: EchoValue) -> EchoValue {
    let Some(bytes) = address.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(address) = String::from_utf8(bytes) else {
        return EchoValue::error();
    };

    match net::connect(address) {
        Ok(connection) => EchoValue::tcp_connection(Box::into_raw(Box::new(connection))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_accept(listener: EchoValue) -> EchoValue {
    let Some(listener) = listener.as_tcp_listener_ref() else {
        return EchoValue::error();
    };

    match net::accept(listener) {
        Ok(connection) => EchoValue::tcp_connection(Box::into_raw(Box::new(connection))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_read(connection: EchoValue, max_bytes: EchoValue) -> EchoValue {
    let Some(connection) = connection.as_tcp_connection_mut() else {
        return EchoValue::error();
    };
    if !max_bytes.is_int() {
        return EchoValue::error();
    }

    match net::read(connection, max_bytes.payload as usize) {
        Ok(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            bytes.into_bytes().to_vec(),
        )))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_write(connection: EchoValue, data: EchoValue) -> EchoValue {
    let Some(connection) = connection.as_tcp_connection_mut() else {
        return EchoValue::error();
    };
    let Some(bytes) = data.string_bytes() else {
        return EchoValue::error();
    };

    match net::write(connection, &bytes) {
        Ok(written) => EchoValue::int(written as i64),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_close(connection: EchoValue) -> EchoValue {
    let Some(connection) = connection.as_tcp_connection_mut() else {
        return EchoValue::error();
    };

    match net::close(connection) {
        Ok(()) => EchoValue::null(),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_http_response_text(body: EchoValue) -> EchoValue {
    let Some(body) = body.string_bytes() else {
        return EchoValue::error();
    };

    let Ok(response) = http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/plain")
        .header(http::header::CONTENT_LENGTH, body.len().to_string())
        .header(http::header::CONNECTION, "close")
        .body(body)
    else {
        return EchoValue::error();
    };

    let (parts, body) = response.into_parts();
    let reason = parts.status.canonical_reason().unwrap_or("OK");
    let mut bytes = format!("HTTP/1.1 {} {reason}\r\n", parts.status.as_u16()).into_bytes();

    for (name, value) in &parts.headers {
        let Ok(value) = value.to_str() else {
            return EchoValue::error();
        };
        bytes.extend_from_slice(name.as_str().as_bytes());
        bytes.extend_from_slice(b": ");
        bytes.extend_from_slice(value.as_bytes());
        bytes.extend_from_slice(b"\r\n");
    }

    bytes.extend_from_slice(b"\r\n");
    bytes.extend_from_slice(&body);

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_exists(name: EchoValue) -> EchoValue {
    match function_reflection_for_value(name) {
        Some(_) => EchoValue::bool(true),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_params(name: EchoValue) -> EchoValue {
    let params = function_reflection_for_value(name)
        .map(|function| function.params_signature)
        .unwrap_or_default();

    echo_runtime_string(params.into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_return_type(name: EchoValue) -> EchoValue {
    let return_type = function_reflection_for_value(name)
        .map(|function| function.return_type)
        .unwrap_or_default();

    echo_runtime_string(return_type.into_bytes())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_reflection_register_function(
    name_ptr: *const u8,
    name_len: usize,
    params_ptr: *const u8,
    params_len: usize,
    return_type_ptr: *const u8,
    return_type_len: usize,
    source_kind: i32,
) {
    let Some(name) = runtime_utf8_arg(name_ptr, name_len) else {
        return;
    };
    let Some(params_signature) = runtime_utf8_arg(params_ptr, params_len) else {
        return;
    };
    let Some(return_type) = runtime_utf8_arg(return_type_ptr, return_type_len) else {
        return;
    };

    let mut functions = function_reflections()
        .lock()
        .expect("function reflection registry should not be poisoned");
    if let Some(existing) = functions.iter_mut().find(|function| {
        function.name.eq_ignore_ascii_case(&name) && function.source_kind == source_kind
    }) {
        existing.params_signature = params_signature;
        existing.return_type = return_type;
    } else {
        functions.push(RuntimeFunctionReflection {
            name,
            params_signature,
            return_type,
            source_kind,
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_type_of(value: EchoValue) -> EchoValue {
    let type_name = match value.kind {
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
    };

    echo_runtime_string(type_name.to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_assert_ok(condition: EchoValue) -> EchoValue {
    let passed = condition.kind == ECHO_VALUE_BOOL && condition.payload != 0;
    record_assertion(passed, "assert.ok failed");
    EchoValue::bool(passed)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_assert_equals(actual: EchoValue, expected: EchoValue) -> EchoValue {
    let passed = echo_values_equal(actual, expected);
    record_assertion(passed, "assert.equals failed");
    EchoValue::bool(passed)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_http_read_request(connection: EchoValue) -> EchoValue {
    if connection.kind != ECHO_VALUE_TCP_CONNECTION {
        return EchoValue::error();
    }

    let Some(connection) =
        (unsafe { (connection.payload as *mut net::EchoTcpConnection).as_mut() })
    else {
        return EchoValue::error();
    };
    let Ok(buffer) = net::read(connection, 4096) else {
        return EchoValue::error();
    };
    let Ok(request) = std::str::from_utf8(buffer.as_bytes()) else {
        return EchoValue::error();
    };
    let Some(request_line) = request.lines().next() else {
        return EchoValue::error();
    };
    let mut parts = request_line.split_whitespace();
    let (Some(_method), Some(path), Some(_version)) = (parts.next(), parts.next(), parts.next())
    else {
        return EchoValue::error();
    };

    let object = echo_value_object_new();
    let path = EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        path.as_bytes().to_vec(),
    ))));
    unsafe { echo_value_object_set(object, b"path".as_ptr(), 4, path) }
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
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_list_new() -> EchoValue {
    EchoValue::list(Box::into_raw(Box::new(EchoList::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_list_append(list: EchoValue, value: EchoValue) -> EchoValue {
    if !list.is_list() {
        return EchoValue::error();
    }

    let Some(values) = (unsafe { (list.payload as *mut EchoList).as_mut() }) else {
        return EchoValue::error();
    };

    values.values.push(value);
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_array_new() -> EchoValue {
    EchoValue::array(Box::into_raw(Box::new(EchoArray::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_array_append(array: EchoValue, value: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(values) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };

    values.keys.push(next_array_append_key(values));
    values.values.push(value);
    array
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_array_set(
    array: EchoValue,
    key: EchoValue,
    value: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(key) = EchoArrayKey::from_value(key) else {
        return EchoValue::error();
    };

    let Some(values) = (unsafe { (array.payload as *mut EchoArray).as_mut() }) else {
        return EchoValue::error();
    };

    if let Some(index) = values.keys.iter().position(|existing| existing == &key) {
        values.values[index] = value;
    } else {
        values.keys.push(key);
        values.values.push(value);
    }
    array
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_index_get(collection: EchoValue, index: EchoValue) -> EchoValue {
    if collection.is_array() {
        let Some(key) = EchoArrayKey::from_value(index) else {
            return EchoValue::null();
        };
        let Some(array) = (unsafe { (collection.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::null();
        };

        return array
            .keys
            .iter()
            .position(|existing| existing == &key)
            .map(|position| array.values[position])
            .unwrap_or_else(EchoValue::null);
    }

    if collection.is_list() {
        let Some(index) = index.int_value() else {
            return EchoValue::null();
        };
        if index < 0 {
            return EchoValue::null();
        }
        let Some(list) = (unsafe { (collection.payload as *const EchoList).as_ref() }) else {
            return EchoValue::null();
        };

        return list
            .values
            .get(index as usize)
            .copied()
            .unwrap_or_else(EchoValue::null);
    }

    EchoValue::error()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_object_new() -> EchoValue {
    EchoValue::object(Box::into_raw(Box::new(EchoObject::new())))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_object_set(
    object: EchoValue,
    field_ptr: *const u8,
    field_len: usize,
    value: EchoValue,
) -> EchoValue {
    if object.kind != ECHO_VALUE_OBJECT || (field_ptr.is_null() && field_len != 0) {
        return EchoValue::error();
    }

    let Some(fields) = (unsafe { (object.payload as *mut EchoObject).as_mut() }) else {
        return EchoValue::error();
    };
    let field_bytes = unsafe { std::slice::from_raw_parts(field_ptr, field_len) };
    let Ok(field) = std::str::from_utf8(field_bytes) else {
        return EchoValue::error();
    };

    fields.fields.push((field.to_string(), value));
    object
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_object_get(
    object: EchoValue,
    field_ptr: *const u8,
    field_len: usize,
) -> EchoValue {
    if object.kind != ECHO_VALUE_OBJECT || (field_ptr.is_null() && field_len != 0) {
        return EchoValue::error();
    }

    let Some(fields) = (unsafe { (object.payload as *const EchoObject).as_ref() }) else {
        return EchoValue::error();
    };
    let field_bytes = unsafe { std::slice::from_raw_parts(field_ptr, field_len) };
    let Ok(field) = std::str::from_utf8(field_bytes) else {
        return EchoValue::error();
    };

    fields
        .fields
        .iter()
        .rev()
        .find_map(|(name, value)| (name == field).then_some(*value))
        .unwrap_or_else(EchoValue::error)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strlen(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::int(bytes.len() as i64),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_count(value: EchoValue) -> EchoValue {
    if value.is_array() {
        let Some(array) = (unsafe { (value.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::error();
        };
        return EchoValue::int(array.values.len() as i64);
    }

    if value.is_list() {
        let Some(list) = (unsafe { (value.payload as *const EchoList).as_ref() }) else {
            return EchoValue::error();
        };
        return EchoValue::int(list.values.len() as i64);
    }

    EchoValue::error()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_values(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        array.values.clone(),
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_keys(
    array: EchoValue,
    filter_value: EchoValue,
    strict: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let strict = strict.bool_value().unwrap_or(false);
    let has_filter = filter_value.kind != ECHO_VALUE_PENDING;

    let mut keys = Vec::new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        if has_filter {
            let matches = if strict {
                echo_values_equal(*value, filter_value)
            } else {
                php_values_equal(*value, filter_value)
            };
            if !matches {
                continue;
            }
        }
        keys.push(key.to_value());
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(keys))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_fill(
    start_index: EchoValue,
    count: EchoValue,
    value: EchoValue,
) -> EchoValue {
    let Some(start_index) = start_index.php_int_value() else {
        return EchoValue::error();
    };
    let Some(count) = count.php_int_value() else {
        return EchoValue::error();
    };
    if !(0..=i32::MAX as i64).contains(&count) {
        return EchoValue::error();
    }

    let mut keys = Vec::with_capacity(count as usize);
    let mut values = Vec::with_capacity(count as usize);
    for offset in 0..count {
        let Some(key) = start_index.checked_add(offset) else {
            return EchoValue::error();
        };
        keys.push(EchoArrayKey::Int(key));
        values.push(value);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_fill_keys(keys: EchoValue, value: EchoValue) -> EchoValue {
    if !keys.is_array() {
        return EchoValue::error();
    }

    let Some(keys_array) = (unsafe { (keys.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut result = echo_value_array_new();
    for key_value in &keys_array.values {
        result = echo_value_array_set(result, *key_value, value);
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_combine(keys: EchoValue, values: EchoValue) -> EchoValue {
    if !keys.is_array() || !values.is_array() {
        return EchoValue::error();
    }

    let Some(keys_array) = (unsafe { (keys.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(values_array) = (unsafe { (values.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    if keys_array.values.len() != values_array.values.len() {
        return EchoValue::error();
    }

    let mut result = echo_value_array_new();
    for (key, value) in keys_array.values.iter().zip(&values_array.values) {
        result = echo_value_array_set(result, *key, *value);
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_pad(
    array: EchoValue,
    length: EchoValue,
    value: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let current_len = array.values.len() as i64;
    let target_len = length.checked_abs().unwrap_or(i64::MAX);

    if target_len > i32::MAX as i64 {
        return EchoValue::error();
    }

    if target_len <= current_len {
        return EchoValue::array(Box::into_raw(Box::new(EchoArray {
            keys: array.keys.clone(),
            values: array.values.clone(),
        })));
    }

    let pad_count = (target_len - current_len) as usize;
    let mut keys = Vec::with_capacity(target_len as usize);
    let mut values = Vec::with_capacity(target_len as usize);
    let mut next_index = 0_i64;

    if length < 0 {
        for _ in 0..pad_count {
            keys.push(EchoArrayKey::Int(next_index));
            values.push(value);
            next_index += 1;
        }
    }

    append_array_pad_values(array, &mut keys, &mut values, &mut next_index);

    if length > 0 {
        for _ in 0..pad_count {
            keys.push(EchoArrayKey::Int(next_index));
            values.push(value);
            next_index += 1;
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

fn append_array_pad_values(
    array: &EchoArray,
    keys: &mut Vec<EchoArrayKey>,
    values: &mut Vec<EchoValue>,
    next_index: &mut i64,
) {
    for (key, value) in array.keys.iter().zip(&array.values) {
        match key {
            EchoArrayKey::Int(_) => {
                keys.push(EchoArrayKey::Int(*next_index));
                *next_index += 1;
            }
            EchoArrayKey::String(bytes) => keys.push(EchoArrayKey::String(bytes.clone())),
        }
        values.push(*value);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_reverse(array: EchoValue, preserve_keys: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let preserve_keys = preserve_keys.bool_value().unwrap_or(false);

    let mut keys = Vec::with_capacity(array.keys.len());
    let mut values = Vec::with_capacity(array.values.len());
    let mut next_index = 0_i64;

    for (key, value) in array.keys.iter().zip(&array.values).rev() {
        let key = if preserve_keys {
            key.clone()
        } else {
            match key {
                EchoArrayKey::Int(_) => {
                    let key = EchoArrayKey::Int(next_index);
                    next_index += 1;
                    key
                }
                EchoArrayKey::String(bytes) => EchoArrayKey::String(bytes.clone()),
            }
        };
        keys.push(key);
        values.push(*value);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_slice(
    array: EchoValue,
    offset: EchoValue,
    length: EchoValue,
    preserve_keys: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(offset) = offset.php_int_value() else {
        return EchoValue::error();
    };

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let array_len = array.values.len() as i64;
    let start = if offset < 0 {
        array_len.saturating_add(offset).max(0)
    } else {
        offset.min(array_len)
    };
    let end = match length.kind {
        ECHO_VALUE_NULL => array_len,
        _ => {
            let Some(length) = length.php_int_value() else {
                return EchoValue::error();
            };
            if length < 0 {
                array_len.saturating_add(length).max(start)
            } else {
                start.saturating_add(length).min(array_len)
            }
        }
    };
    let preserve_keys = preserve_keys.bool_value().unwrap_or(false);

    let mut keys = Vec::with_capacity((end - start) as usize);
    let mut values = Vec::with_capacity((end - start) as usize);
    let mut next_index = 0_i64;

    for index in start as usize..end as usize {
        let key = match &array.keys[index] {
            EchoArrayKey::Int(_) if !preserve_keys => {
                let key = EchoArrayKey::Int(next_index);
                next_index += 1;
                key
            }
            key => key.clone(),
        };
        keys.push(key);
        values.push(array.values[index]);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_chunk(
    array: EchoValue,
    length: EchoValue,
    preserve_keys: EchoValue,
) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    if length < 1 {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let preserve_keys = preserve_keys.bool_value().unwrap_or(false);

    let chunk_count = array.values.len().div_ceil(length as usize);
    let mut outer_values = Vec::with_capacity(chunk_count);

    for start in (0..array.values.len()).step_by(length as usize) {
        let end = start
            .saturating_add(length as usize)
            .min(array.values.len());
        let mut chunk_keys = Vec::with_capacity(end - start);
        let mut chunk_values = Vec::with_capacity(end - start);

        for (offset, index) in (start..end).enumerate() {
            let key = if preserve_keys {
                array.keys[index].clone()
            } else {
                EchoArrayKey::Int(offset as i64)
            };
            chunk_keys.push(key);
            chunk_values.push(array.values[index]);
        }

        outer_values.push(EchoValue::array(Box::into_raw(Box::new(EchoArray {
            keys: chunk_keys,
            values: chunk_values,
        }))));
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        outer_values,
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_merge(arrays: EchoValue) -> EchoValue {
    if !arrays.is_array() {
        return EchoValue::error();
    }

    let Some(arrays) = (unsafe { (arrays.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut next_index = 0_i64;

    for array_value in &arrays.values {
        if !array_value.is_array() {
            return EchoValue::error();
        }
        let Some(array) = (unsafe { (array_value.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::error();
        };

        for (key, value) in array.keys.iter().zip(&array.values) {
            match key {
                EchoArrayKey::Int(_) => {
                    keys.push(EchoArrayKey::Int(next_index));
                    values.push(*value);
                    next_index += 1;
                }
                EchoArrayKey::String(bytes) => {
                    let key = EchoArrayKey::String(bytes.clone());
                    match keys.iter().position(|existing| existing == &key) {
                        Some(index) => values[index] = *value,
                        None => {
                            keys.push(key);
                            values.push(*value);
                        }
                    }
                }
            }
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_replace(arrays: EchoValue) -> EchoValue {
    if !arrays.is_array() {
        return EchoValue::error();
    }

    let Some(arrays) = (unsafe { (arrays.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    if arrays.values.is_empty() {
        return EchoValue::error();
    }

    let mut result = echo_value_array_new();
    for array_value in &arrays.values {
        if !array_value.is_array() {
            return EchoValue::error();
        }
        let Some(array) = (unsafe { (array_value.payload as *const EchoArray).as_ref() }) else {
            return EchoValue::error();
        };

        for (key, value) in array.keys.iter().zip(&array.values) {
            result = echo_value_array_set(result, key.to_value(), *value);
        }
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_flip(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut result = echo_value_array_new();
    for (key, value) in array.keys.iter().zip(&array.values) {
        if !matches!(value.kind, ECHO_VALUE_INT | ECHO_VALUE_STRING) {
            continue;
        }
        result = echo_value_array_set(result, *value, key.to_value());
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_count_values(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut keys: Vec<EchoArrayKey> = Vec::new();
    let mut values: Vec<EchoValue> = Vec::new();
    for value in &array.values {
        if !matches!(value.kind, ECHO_VALUE_INT | ECHO_VALUE_STRING) {
            continue;
        }
        let Some(key) = EchoArrayKey::from_value(*value) else {
            continue;
        };
        match keys.iter().position(|existing| existing == &key) {
            Some(index) => values[index] = EchoValue::int(values[index].payload as i64 + 1),
            None => {
                keys.push(key);
                values.push(EchoValue::int(1));
            }
        }
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_key_exists(key: EchoValue, array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(key) = EchoArrayKey::from_value(key) else {
        return EchoValue::error();
    };
    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    EchoValue::bool(array.keys.contains(&key))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_key_first(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    match array.keys.first() {
        Some(key) => key.to_value(),
        None => EchoValue::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_key_last(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    match array.keys.last() {
        Some(key) => key.to_value(),
        None => EchoValue::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_in_array(
    needle: EchoValue,
    haystack: EchoValue,
    strict: EchoValue,
) -> EchoValue {
    if !haystack.is_array() {
        return EchoValue::error();
    }

    let Some(haystack) = (unsafe { (haystack.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let strict = strict.bool_value().unwrap_or(false);

    EchoValue::bool(haystack.values.iter().any(|value| {
        if strict {
            echo_values_equal(needle, *value)
        } else {
            php_values_equal(needle, *value)
        }
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_search(
    needle: EchoValue,
    haystack: EchoValue,
    strict: EchoValue,
) -> EchoValue {
    if !haystack.is_array() {
        return EchoValue::error();
    }

    let Some(haystack) = (unsafe { (haystack.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let strict = strict.bool_value().unwrap_or(false);

    for (key, value) in haystack.keys.iter().zip(&haystack.values) {
        let matches = if strict {
            echo_values_equal(needle, *value)
        } else {
            php_values_equal(needle, *value)
        };
        if matches {
            return key.to_value();
        }
    }

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_sum(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut total = PhpNumber::Int(0);
    for value in &array.values {
        total = php_number_add(
            total,
            PhpNumber::coerce(*value).unwrap_or(PhpNumber::Int(0)),
        );
    }

    total.into_echo_value()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_array_product(array: EchoValue) -> EchoValue {
    if !array.is_array() {
        return EchoValue::error();
    }

    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut product = PhpNumber::Int(1);
    for value in &array.values {
        product = php_number_mul(
            product,
            PhpNumber::coerce(*value).unwrap_or(PhpNumber::Int(0)),
        );
    }

    product.into_echo_value()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_function_exists(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => match std::str::from_utf8(&bytes) {
            Ok(name) => EchoValue::bool(
                function_reflection_by_name_and_source(name, REFLECTION_SOURCE_PHP_BUILTIN)
                    .is_some(),
            ),
            Err(_) => EchoValue::bool(false),
        },
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gettype(value: EchoValue) -> EchoValue {
    let type_name = match value.kind {
        ECHO_VALUE_NULL => b"NULL".as_slice(),
        ECHO_VALUE_BOOL => b"boolean".as_slice(),
        ECHO_VALUE_INT => b"integer".as_slice(),
        ECHO_VALUE_FLOAT => b"double".as_slice(),
        ECHO_VALUE_STRING => b"string".as_slice(),
        ECHO_VALUE_ARRAY => b"array".as_slice(),
        ECHO_VALUE_LIST => b"list".as_slice(),
        ECHO_VALUE_TASK
        | ECHO_VALUE_TASK_GROUP
        | ECHO_VALUE_OBJECT
        | ECHO_VALUE_PROCESS
        | ECHO_VALUE_THREAD => b"object".as_slice(),
        ECHO_VALUE_TCP_LISTENER | ECHO_VALUE_TCP_CONNECTION => b"resource".as_slice(),
        _ => b"unknown type".as_slice(),
    };
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(type_name.to_vec()))))
}

fn echo_runtime_string(bytes: Vec<u8>) -> EchoValue {
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
}

fn function_reflection_for_value(value: EchoValue) -> Option<RuntimeFunctionReflection> {
    let bytes = value.string_bytes()?;
    let name = std::str::from_utf8(&bytes).ok()?;
    function_reflections()
        .lock()
        .expect("function reflection registry should not be poisoned")
        .iter()
        .find(|function| function.name.eq_ignore_ascii_case(name))
        .cloned()
}

fn function_reflection_by_name_and_source(
    name: &str,
    source_kind: i32,
) -> Option<RuntimeFunctionReflection> {
    function_reflections()
        .lock()
        .expect("function reflection registry should not be poisoned")
        .iter()
        .find(|function| {
            function.name.eq_ignore_ascii_case(name) && function.source_kind == source_kind
        })
        .cloned()
}

fn function_reflection_by_name(name: &str) -> Option<RuntimeFunctionReflection> {
    function_reflections()
        .lock()
        .expect("function reflection registry should not be poisoned")
        .iter()
        .find(|function| function.name.eq_ignore_ascii_case(name))
        .cloned()
}

fn function_reflections() -> &'static Mutex<Vec<RuntimeFunctionReflection>> {
    static FUNCTION_REFLECTIONS: OnceLock<Mutex<Vec<RuntimeFunctionReflection>>> = OnceLock::new();
    FUNCTION_REFLECTIONS.get_or_init(|| Mutex::new(Vec::new()))
}

fn runtime_utf8_arg(ptr: *const u8, len: usize) -> Option<String> {
    if ptr.is_null() && len != 0 {
        return None;
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(bytes)
        .ok()
        .map(std::string::ToString::to_string)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_array(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array())
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
pub extern "C" fn echo_php_array_is_list(value: EchoValue) -> EchoValue {
    if !value.is_array() {
        return EchoValue::bool(false);
    }
    let Some(array) = (unsafe { (value.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    EchoValue::bool(
        array
            .keys
            .iter()
            .enumerate()
            .all(|(index, key)| key == &EchoArrayKey::Int(index as i64)),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_countable(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array() || value.is_list())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_iterable(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array() || value.is_list())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_null(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_null())
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
pub extern "C" fn echo_php_is_bool(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_bool())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_callable(value: EchoValue) -> EchoValue {
    if value.kind != ECHO_VALUE_STRING {
        return EchoValue::bool(false);
    }

    let Some(string) = (unsafe { (value.payload as *const EchoString).as_ref() }) else {
        return EchoValue::bool(false);
    };

    match std::str::from_utf8(&string.bytes) {
        Ok(name) => EchoValue::bool(function_reflection_by_name(name).is_some()),
        Err(_) => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_int(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_int())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_float(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_float())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_finite(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::bool(value.is_finite()),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_infinite(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::bool(value.is_infinite()),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_nan(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::bool(value.is_nan()),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_object(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_object())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_resource(value: EchoValue) -> EchoValue {
    EchoValue::bool(matches!(
        value.kind,
        ECHO_VALUE_TCP_LISTENER
            | ECHO_VALUE_TCP_CONNECTION
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_THREAD
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_string(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_scalar(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_bool() || value.is_int() || value.is_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strval(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes)))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_boolval(value: EchoValue) -> EchoValue {
    match value.bool_value() {
        Some(value) => EchoValue::bool(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_intval(value: EchoValue) -> EchoValue {
    match value.php_int_value() {
        Some(value) => EchoValue::int(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_floatval(value: EchoValue) -> EchoValue {
    match php_float_cast(value) {
        Some(value) => EchoValue::float(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strtoupper(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            bytes.make_ascii_uppercase();
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strtolower(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            bytes.make_ascii_lowercase();
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ucwords(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            let mut uppercase_next = true;
            for byte in &mut bytes {
                if uppercase_next {
                    byte.make_ascii_uppercase();
                }
                uppercase_next = matches!(*byte, b' ' | b'\t' | b'\r' | b'\n' | 0x0c | 0x0b);
            }
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strrev(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            bytes.reverse();
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ucfirst(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            if let Some(first) = bytes.first_mut() {
                first.make_ascii_uppercase();
            }
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_lcfirst(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            if let Some(first) = bytes.first_mut() {
                first.make_ascii_lowercase();
            }
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
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
    match value.string_bytes() {
        Some(mut bytes) => {
            for byte in &mut bytes {
                *byte = match *byte {
                    b'a'..=b'm' | b'A'..=b'M' => *byte + 13,
                    b'n'..=b'z' | b'N'..=b'Z' => *byte - 13,
                    other => other,
                };
            }
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_chr(value: EchoValue) -> EchoValue {
    match value.int_value() {
        Some(codepoint) => {
            let byte = codepoint.rem_euclid(256) as u8;
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(vec![byte]))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_dechex(value: EchoValue) -> EchoValue {
    match value.php_int_value() {
        Some(number) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            format!("{:x}", number as u64).into_bytes(),
        )))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_decbin(value: EchoValue) -> EchoValue {
    match value.php_int_value() {
        Some(number) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            format!("{:b}", number as u64).into_bytes(),
        )))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_decoct(value: EchoValue) -> EchoValue {
    match value.php_int_value() {
        Some(number) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            format!("{:o}", number as u64).into_bytes(),
        )))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_bindec(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => php_unsigned_base_to_decimal(&bytes, 2),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hexdec(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => php_unsigned_base_to_decimal(&bytes, 16),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_octdec(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => php_unsigned_base_to_decimal(&bytes, 8),
        None => EchoValue::error(),
    }
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_deg2rad(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(value.to_radians()),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rad2deg(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(value.to_degrees()),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sin(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_sin(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cos(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_cos(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_tan(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_sin(value) / echo_math_cos(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_asin(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_asin(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_acos(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_acos(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_atan(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_atan(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_atan2(y: EchoValue, x: EchoValue) -> EchoValue {
    match (php_float_coercion(y), php_float_coercion(x)) {
        (Some(y), Some(x)) => EchoValue::float(echo_math_atan2(y, x)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sinh(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_sinh(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cosh(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_cosh(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_tanh(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_tanh(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_asinh(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_asinh(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_acosh(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_acosh(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_atanh(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_atanh(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ceil(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_ceil(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_floor(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_floor(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sqrt(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_sqrt(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_exp(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_exp(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_expm1(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_expm1(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_log(value: EchoValue, base: EchoValue) -> EchoValue {
    match (php_float_coercion(value), php_float_coercion(base)) {
        (Some(value), Some(base)) if base > 0.0 => {
            EchoValue::float(echo_math_ln(value) / echo_math_ln(base))
        }
        (Some(_), Some(_)) => EchoValue::error(),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_log10(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_ln(value) / std::f64::consts::LN_10),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_log1p(value: EchoValue) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(echo_math_log1p(value)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_pow(base: EchoValue, exponent: EchoValue) -> EchoValue {
    echo_value_pow(base, exponent)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fdiv(num1: EchoValue, num2: EchoValue) -> EchoValue {
    match (php_float_coercion(num1), php_float_coercion(num2)) {
        (Some(num1), Some(num2)) => EchoValue::float(num1 / num2),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fpow(base: EchoValue, exponent: EchoValue) -> EchoValue {
    match (php_float_coercion(base), php_float_coercion(exponent)) {
        (Some(base), Some(exponent)) => EchoValue::float(echo_math_pow_float(base, exponent)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hypot(x: EchoValue, y: EchoValue) -> EchoValue {
    match (php_float_coercion(x), php_float_coercion(y)) {
        (Some(x), Some(y)) => EchoValue::float(echo_math_sqrt(x * x + y * y)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_pi() -> EchoValue {
    EchoValue::float(std::f64::consts::PI)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fmod(num1: EchoValue, num2: EchoValue) -> EchoValue {
    match (php_float_coercion(num1), php_float_coercion(num2)) {
        (Some(num1), Some(num2)) => EchoValue::float(num1 % num2),
        _ => EchoValue::error(),
    }
}

fn echo_math_sin(value: f64) -> f64 {
    if !value.is_finite() {
        return f64::NAN;
    }

    let (x, sign) = reduce_to_half_pi(value);
    let x2 = x * x;
    sign * x
        * (1.0
            + x2 * (-1.0 / 6.0
                + x2 * (1.0 / 120.0
                    + x2 * (-1.0 / 5040.0
                        + x2 * (1.0 / 362880.0 + x2 * (-1.0 / 39916800.0 + x2 / 6227020800.0))))))
}

fn echo_math_cos(value: f64) -> f64 {
    echo_math_sin(std::f64::consts::FRAC_PI_2 - value)
}

fn echo_math_asin(value: f64) -> f64 {
    if !(-1.0..=1.0).contains(&value) {
        return f64::NAN;
    }
    echo_math_atan2(value, echo_math_sqrt(1.0 - value * value))
}

fn echo_math_acos(value: f64) -> f64 {
    if !(-1.0..=1.0).contains(&value) {
        return f64::NAN;
    }
    echo_math_atan2(echo_math_sqrt(1.0 - value * value), value)
}

fn echo_math_atan(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return value.signum() * std::f64::consts::FRAC_PI_2;
    }

    let sign = if value < 0.0 { -1.0 } else { 1.0 };
    sign * echo_math_atan_positive(value.abs())
}

fn echo_math_atan2(y: f64, x: f64) -> f64 {
    if y.is_nan() || x.is_nan() {
        return f64::NAN;
    }

    if x > 0.0 {
        echo_math_atan(y / x)
    } else if x < 0.0 && y >= 0.0 {
        echo_math_atan(y / x) + std::f64::consts::PI
    } else if x < 0.0 {
        echo_math_atan(y / x) - std::f64::consts::PI
    } else if y > 0.0 {
        std::f64::consts::FRAC_PI_2
    } else if y < 0.0 {
        -std::f64::consts::FRAC_PI_2
    } else {
        0.0
    }
}

fn echo_math_atan_positive(value: f64) -> f64 {
    if value > 1.0 {
        return std::f64::consts::FRAC_PI_2 - echo_math_atan_positive(1.0 / value);
    }
    if value > 0.41421356237309503 {
        return std::f64::consts::FRAC_PI_4 + echo_math_atan_kernel((value - 1.0) / (value + 1.0));
    }

    echo_math_atan_kernel(value)
}

fn echo_math_atan_kernel(value: f64) -> f64 {
    let x2 = value * value;
    value
        * (1.0
            + x2 * (-1.0 / 3.0
                + x2 * (1.0 / 5.0
                    + x2 * (-1.0 / 7.0
                        + x2 * (1.0 / 9.0
                            + x2 * (-1.0 / 11.0
                                + x2 * (1.0 / 13.0 + x2 * (-1.0 / 15.0 + x2 / 17.0))))))))
}

fn echo_math_sinh(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return value;
    }

    let abs = value.abs();
    let result = if abs < 0.00000001 {
        value
    } else {
        let exp = echo_math_exp(abs);
        0.5 * (exp - 1.0 / exp)
    };
    result.copysign(value)
}

fn echo_math_cosh(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return f64::INFINITY;
    }

    let exp = echo_math_exp(value.abs());
    0.5 * (exp + 1.0 / exp)
}

fn echo_math_tanh(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value == 0.0 {
        return value;
    }
    if value.is_infinite() {
        return value.signum();
    }

    let exp2 = echo_math_exp(2.0 * value.abs());
    let result = (exp2 - 1.0) / (exp2 + 1.0);
    result.copysign(value)
}

fn echo_math_asinh(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        return value;
    }
    let abs = value.abs();
    let result = echo_math_ln(abs + echo_math_sqrt(abs * abs + 1.0));
    result.copysign(value)
}

fn echo_math_acosh(value: f64) -> f64 {
    if value < 1.0 || value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return f64::INFINITY;
    }

    echo_math_ln(value + echo_math_sqrt(value - 1.0) * echo_math_sqrt(value + 1.0))
}

fn echo_math_atanh(value: f64) -> f64 {
    if value.is_nan() || value < -1.0 || value > 1.0 {
        return f64::NAN;
    }
    if value == 1.0 {
        return f64::INFINITY;
    }
    if value == -1.0 {
        return f64::NEG_INFINITY;
    }

    0.5 * echo_math_ln((1.0 + value) / (1.0 - value))
}

fn echo_math_exp(value: f64) -> f64 {
    const LN_2: f64 = std::f64::consts::LN_2;
    const MAX_EXP_INPUT: f64 = 709.782712893384;
    const MIN_EXP_INPUT: f64 = -745.1332191019411;

    if value.is_nan() {
        return f64::NAN;
    }
    if value > MAX_EXP_INPUT {
        return f64::INFINITY;
    }
    if value < MIN_EXP_INPUT {
        return 0.0;
    }
    if value == 0.0 {
        return 1.0;
    }

    let scaled = value / LN_2;
    let k = if scaled >= 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    };
    let r = value - (k as f64) * LN_2;
    echo_math_pow2(k) * echo_math_exp_kernel(r)
}

fn echo_math_expm1(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value == 0.0 {
        return value;
    }
    if value.abs() >= 0.000001 {
        return echo_math_exp(value) - 1.0;
    }

    let mut term = value;
    let mut sum = value;
    for n in 2..=24 {
        term *= value / n as f64;
        sum += term;
    }
    sum
}

fn echo_math_exp_kernel(value: f64) -> f64 {
    let mut term = 1.0;
    let mut sum = 1.0;
    for n in 1..=24 {
        term *= value / n as f64;
        sum += term;
    }
    sum
}

fn echo_math_pow2(exp: i32) -> f64 {
    if exp > 1023 {
        return f64::INFINITY;
    }
    if exp < -1074 {
        return 0.0;
    }
    if exp >= -1022 {
        return f64::from_bits(((exp + 1023) as u64) << 52);
    }
    f64::from_bits(1_u64 << (exp + 1074))
}

fn echo_math_ln(value: f64) -> f64 {
    const LN_2: f64 = std::f64::consts::LN_2;

    if value.is_nan() || value < 0.0 {
        return f64::NAN;
    }
    if value == 0.0 {
        return f64::NEG_INFINITY;
    }
    if value.is_infinite() {
        return f64::INFINITY;
    }

    let (mantissa, exponent) = echo_math_frexp(value);
    let y = (mantissa - 1.0) / (mantissa + 1.0);
    let y2 = y * y;
    let mut term = y;
    let mut sum = 0.0;
    let mut denominator = 1.0;

    for _ in 0..48 {
        sum += term / denominator;
        term *= y2;
        denominator += 2.0;
    }

    2.0 * sum + (exponent as f64) * LN_2
}

fn echo_math_log1p(value: f64) -> f64 {
    if value.is_nan() || value < -1.0 {
        return f64::NAN;
    }
    if value == -1.0 {
        return f64::NEG_INFINITY;
    }
    if value.abs() >= 0.000001 {
        return echo_math_ln(1.0 + value);
    }

    let mut term = value;
    let mut sum = value;
    for n in 2..=48 {
        term *= -value;
        sum += term / n as f64;
    }
    sum
}

fn echo_math_pow_float(base: f64, exponent: f64) -> f64 {
    if base.is_nan() || exponent.is_nan() {
        return f64::NAN;
    }
    if exponent == 0.0 {
        return 1.0;
    }
    if base == 0.0 && exponent < 0.0 {
        return f64::INFINITY;
    }
    if base < 0.0 && exponent.fract() != 0.0 {
        return f64::NAN;
    }
    if base == 0.0 {
        return 0.0;
    }

    let magnitude = echo_math_exp(echo_math_ln(base.abs()) * exponent);
    if base < 0.0 && (exponent as i64) % 2 != 0 {
        -magnitude
    } else {
        magnitude
    }
}

fn echo_math_frexp(value: f64) -> (f64, i32) {
    let bits = value.to_bits();
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction_bits = bits & 0x000f_ffff_ffff_ffff;

    if exponent_bits == 0 {
        let (mantissa, exponent) = echo_math_frexp(value * echo_math_pow2(52));
        return (mantissa, exponent - 52);
    }

    let mantissa_bits = (1023_u64 << 52) | fraction_bits;
    (f64::from_bits(mantissa_bits), exponent_bits - 1023)
}

fn echo_math_sqrt(value: f64) -> f64 {
    if value < 0.0 {
        return f64::NAN;
    }
    if value == 0.0 || value.is_infinite() {
        return value;
    }

    let mut estimate = if value >= 1.0 { value } else { 1.0 };
    for _ in 0..24 {
        estimate = 0.5 * (estimate + value / estimate);
    }
    estimate
}

fn echo_math_floor(value: f64) -> f64 {
    if !value.is_finite() || value.abs() >= i64::MAX as f64 {
        return value;
    }

    let truncated = value as i64 as f64;
    if value < truncated {
        truncated - 1.0
    } else {
        truncated
    }
}

fn echo_math_ceil(value: f64) -> f64 {
    if !value.is_finite() || value.abs() >= i64::MAX as f64 {
        return value;
    }

    let truncated = value as i64 as f64;
    if value < 0.0 && truncated == 0.0 {
        -0.0
    } else if value > truncated {
        truncated + 1.0
    } else {
        truncated
    }
}

fn reduce_to_half_pi(value: f64) -> (f64, f64) {
    let mut x = value - (value / std::f64::consts::TAU) as i64 as f64 * std::f64::consts::TAU;
    if x > std::f64::consts::PI {
        x -= std::f64::consts::TAU;
    } else if x < -std::f64::consts::PI {
        x += std::f64::consts::TAU;
    }

    if x > std::f64::consts::FRAC_PI_2 {
        (std::f64::consts::PI - x, 1.0)
    } else if x < -std::f64::consts::FRAC_PI_2 {
        (-std::f64::consts::PI - x, -1.0)
    } else {
        (x, 1.0)
    }
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

fn lowercase_hex_bytes(bytes: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize]);
        encoded.push(HEX[(byte & 0x0f) as usize]);
    }
    encoded
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_base64_encode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(encode_base64(
            &bytes,
        ))))),
        None => EchoValue::error(),
    }
}

fn encode_base64(bytes: &[u8]) -> Vec<u8> {
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_base64_decode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            decode_base64_non_strict(&bytes),
        )))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rawurlencode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(percent_encode(&bytes, PercentEncodingMode::RawUrl)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_urlencode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(percent_encode(&bytes, PercentEncodingMode::FormUrl)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rawurldecode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(percent_decode(&bytes, false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_urldecode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(percent_decode(&bytes, true)),
        None => EchoValue::error(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PercentEncodingMode {
    RawUrl,
    FormUrl,
}

fn percent_encode(bytes: &[u8], mode: PercentEncodingMode) -> Vec<u8> {
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

fn percent_encode_keeps_byte(byte: u8, mode: PercentEncodingMode) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(byte, b'-' | b'_' | b'.')
        || (mode == PercentEncodingMode::RawUrl && byte == b'~')
}

fn percent_decode(bytes: &[u8], decode_plus: bool) -> Vec<u8> {
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
pub extern "C" fn echo_php_escapeshellarg(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            escape_shell_arg_unix(&bytes),
        )))),
        None => EchoValue::error(),
    }
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
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            escape_shell_cmd_unix(&bytes),
        )))),
        None => EchoValue::error(),
    }
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_explode(
    separator: EchoValue,
    string: EchoValue,
    limit: EchoValue,
) -> EchoValue {
    let Some(separator) = separator.string_bytes() else {
        return EchoValue::error();
    };
    let Some(string) = string.string_bytes() else {
        return EchoValue::error();
    };
    let Some(limit) = limit.php_int_value() else {
        return EchoValue::error();
    };
    if separator.is_empty() {
        return EchoValue::error();
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        explode_bytes(&separator, &string, limit)
            .into_iter()
            .map(|bytes| EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes)))))
            .collect(),
    ))))
}

fn explode_bytes(separator: &[u8], string: &[u8], limit: i64) -> Vec<Vec<u8>> {
    let limit = if limit == 0 { 1 } else { limit };
    if limit > 0 {
        return explode_bytes_positive_limit(separator, string, limit as usize);
    }

    let mut parts = explode_bytes_all(separator, string);
    let omit = limit.unsigned_abs() as usize;
    let keep = parts.len().saturating_sub(omit);
    parts.truncate(keep);
    parts
}

fn explode_bytes_positive_limit(separator: &[u8], string: &[u8], limit: usize) -> Vec<Vec<u8>> {
    if limit == 1 {
        return vec![string.to_vec()];
    }

    let mut parts = Vec::new();
    let mut start = 0;
    while parts.len() + 1 < limit {
        let Some(offset) = find_subslice(&string[start..], separator) else {
            break;
        };
        let end = start + offset;
        parts.push(string[start..end].to_vec());
        start = end + separator.len();
    }
    parts.push(string[start..].to_vec());
    parts
}

fn explode_bytes_all(separator: &[u8], string: &[u8]) -> Vec<Vec<u8>> {
    let mut parts = Vec::new();
    let mut start = 0;
    while let Some(offset) = find_subslice(&string[start..], separator) {
        let end = start + offset;
        parts.push(string[start..end].to_vec());
        start = end + separator.len();
    }
    parts.push(string[start..].to_vec());
    parts
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_implode(separator: EchoValue, array: EchoValue) -> EchoValue {
    let Some(separator) = separator.string_bytes() else {
        return EchoValue::error();
    };
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut joined = Vec::new();
    for (index, value) in array.values.iter().enumerate() {
        if index > 0 {
            joined.extend_from_slice(&separator);
        }
        let Some(bytes) = value.string_bytes() else {
            return EchoValue::error();
        };
        joined.extend_from_slice(&bytes);
    }

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(joined))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_file_exists(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_exists(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_chdir(directory: EchoValue) -> EchoValue {
    match directory.string_bytes() {
        Some(bytes) => EchoValue::bool(path_chdir(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getcwd() -> EchoValue {
    path_getcwd()
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getenv(name: EchoValue, _local_only: EchoValue) -> EchoValue {
    if name.kind == ECHO_VALUE_NULL {
        let mut result = echo_value_array_new();

        for (key, value) in env::vars_os() {
            result = echo_value_array_set(
                result,
                echo_runtime_string(os_string_bytes(&key)),
                echo_runtime_string(os_string_bytes(&value)),
            );
        }

        return result;
    }

    let Some(bytes) = name.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(key) = String::from_utf8(bytes) else {
        return EchoValue::bool(false);
    };

    env::var_os(key)
        .map(|value| echo_runtime_string(os_string_bytes(&value)))
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gethostname() -> EchoValue {
    env::var_os("HOSTNAME")
        .and_then(non_empty_os_string_bytes)
        .or_else(|| hostname_file_bytes(Path::new("/proc/sys/kernel/hostname")))
        .or_else(|| hostname_file_bytes(Path::new("/etc/hostname")))
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getmypid() -> EchoValue {
    EchoValue::int(std::process::id() as i64)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_putenv(assignment: EchoValue) -> EchoValue {
    let Some(bytes) = assignment.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(assignment) = String::from_utf8(bytes) else {
        return EchoValue::bool(false);
    };

    if let Some((name, value)) = assignment.split_once('=') {
        if name.is_empty() {
            return EchoValue::bool(false);
        }

        unsafe {
            env::set_var(name, value);
        }
    } else {
        if assignment.is_empty() {
            return EchoValue::bool(false);
        }

        unsafe {
            env::remove_var(assignment);
        }
    }

    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_define(name: EchoValue, _value: EchoValue) -> EchoValue {
    match name.string_bytes() {
        Some(bytes) if !bytes.is_empty() => EchoValue::bool(true),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_microtime(as_float: EchoValue) -> EchoValue {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));

    if as_float.bool_value().unwrap_or(false) {
        return EchoValue::float(now.as_secs_f64());
    }

    let micros = now.subsec_micros();
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        format!("0.{micros:06} {}", now.as_secs()).into_bytes(),
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_require(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => require_path(&bytes),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_require_once(filename: EchoValue) -> EchoValue {
    let Some(bytes) = filename.string_bytes() else {
        return EchoValue::error();
    };

    let key = canonical_require_key(&bytes);
    let files = REQUIRED_ONCE_FILES.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut files = files.lock().expect("require_once set poisoned");
        if files.contains(&key) {
            return EchoValue::bool(true);
        }
        files.insert(key);
    }

    require_path(&bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_dir(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_is_dir(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_file(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_is_file(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_link(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_is_link(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_readable(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_is_readable(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_writable(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_is_writable(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_executable(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_is_executable(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filesize(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_filesize(&bytes)
            .and_then(|size| i64::try_from(size).ok())
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileatime(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_fileatime(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filectime(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_filectime(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filemtime(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_filemtime(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileinode(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_fileinode(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileowner(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_fileowner(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filegroup(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_filegroup(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileperms(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_fileperms(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filetype(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_filetype(&bytes)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_readlink(path: EchoValue) -> EchoValue {
    match path.string_bytes() {
        Some(bytes) => path_readlink(&bytes)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_link(target: EchoValue, link: EchoValue) -> EchoValue {
    match (target.string_bytes(), link.string_bytes()) {
        (Some(target), Some(link)) => EchoValue::bool(path_link(&target, &link)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_symlink(target: EchoValue, link: EchoValue) -> EchoValue {
    match (target.string_bytes(), link.string_bytes()) {
        (Some(target), Some(link)) => EchoValue::bool(path_symlink(&target, &link)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sys_get_temp_dir() -> EchoValue {
    path_bytes(env::temp_dir())
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::string(Box::into_raw(Box::new(EchoString::new(Vec::new())))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_tempnam(directory: EchoValue, prefix: EchoValue) -> EchoValue {
    match (directory.string_bytes(), prefix.string_bytes()) {
        (Some(directory), Some(prefix)) => path_tempnam(&directory, &prefix)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_uniqid(prefix: EchoValue, more_entropy: EchoValue) -> EchoValue {
    let Some(prefix) = prefix.string_bytes() else {
        return EchoValue::error();
    };
    let more_entropy = more_entropy.bool_value().unwrap_or(false);

    echo_runtime_string(php_uniqid(&prefix, more_entropy))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_touch(
    filename: EchoValue,
    mtime: EchoValue,
    atime: EchoValue,
) -> EchoValue {
    let Some(bytes) = filename.string_bytes() else {
        return EchoValue::error();
    };
    let mtime = if mtime.is_null() {
        None
    } else {
        mtime.php_int_value()
    };
    let atime = if atime.is_null() {
        None
    } else {
        atime.php_int_value()
    };

    EchoValue::bool(path_touch(&bytes, mtime, atime))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_copy(from: EchoValue, to: EchoValue, _context: EchoValue) -> EchoValue {
    match (from.string_bytes(), to.string_bytes()) {
        (Some(from), Some(to)) => EchoValue::bool(path_copy(&from, &to)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rename(
    from: EchoValue,
    to: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    match (from.string_bytes(), to.string_bytes()) {
        (Some(from), Some(to)) => EchoValue::bool(path_rename(&from, &to)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_unlink(filename: EchoValue, _context: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_unlink(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_mkdir(
    directory: EchoValue,
    permissions: EchoValue,
    recursive: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(bytes) = directory.string_bytes() else {
        return EchoValue::error();
    };
    let permissions = permissions.php_int_value().unwrap_or(0o777);
    let recursive = recursive.bool_value().unwrap_or(false);

    EchoValue::bool(path_mkdir(&bytes, permissions, recursive))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rmdir(directory: EchoValue, _context: EchoValue) -> EchoValue {
    match directory.string_bytes() {
        Some(bytes) => EchoValue::bool(path_rmdir(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_realpath(path: EchoValue) -> EchoValue {
    match path.string_bytes() {
        Some(bytes) => path_realpath(&bytes)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[cfg(unix)]
fn path_exists(bytes: &[u8]) -> bool {
    Path::new(OsStr::from_bytes(bytes)).exists()
}

#[cfg(unix)]
fn path_chdir(bytes: &[u8]) -> bool {
    env::set_current_dir(Path::new(OsStr::from_bytes(bytes))).is_ok()
}

#[cfg(unix)]
fn path_getcwd() -> Option<Vec<u8>> {
    env::current_dir()
        .ok()
        .map(|path| path.into_os_string().as_bytes().to_vec())
}

#[cfg(unix)]
fn require_path(bytes: &[u8]) -> EchoValue {
    let path = Path::new(OsStr::from_bytes(bytes));
    if path.exists() {
        EchoValue::bool(true)
    } else {
        eprintln!(
            "PHP Fatal error: Failed opening required '{}'",
            String::from_utf8_lossy(bytes)
        );
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn canonical_require_key(bytes: &[u8]) -> Vec<u8> {
    let path = Path::new(OsStr::from_bytes(bytes));
    std::fs::canonicalize(path)
        .ok()
        .map(|path| path.as_os_str().as_bytes().to_vec())
        .unwrap_or_else(|| bytes.to_vec())
}

#[cfg(not(unix))]
fn path_exists(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(|path| Path::new(path).exists())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn path_chdir(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(Path::new)
        .map(|path| env::set_current_dir(path).is_ok())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn path_getcwd() -> Option<Vec<u8>> {
    env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().as_bytes().to_vec())
}

#[cfg(not(unix))]
fn require_path(bytes: &[u8]) -> EchoValue {
    let Ok(path) = std::str::from_utf8(bytes) else {
        return EchoValue::error();
    };
    if Path::new(path).exists() {
        EchoValue::bool(true)
    } else {
        eprintln!("PHP Fatal error: Failed opening required '{path}'");
        std::process::exit(1);
    }
}

#[cfg(not(unix))]
fn canonical_require_key(bytes: &[u8]) -> Vec<u8> {
    let Ok(path) = std::str::from_utf8(bytes) else {
        return bytes.to_vec();
    };
    std::fs::canonicalize(path)
        .ok()
        .and_then(|path| path.into_os_string().into_string().ok())
        .map(String::into_bytes)
        .unwrap_or_else(|| bytes.to_vec())
}

#[cfg(unix)]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().as_bytes().to_vec()
}

fn non_empty_os_string_bytes(value: std::ffi::OsString) -> Option<Vec<u8>> {
    let bytes = os_string_bytes(&value);

    if bytes.is_empty() { None } else { Some(bytes) }
}

fn hostname_file_bytes(path: &Path) -> Option<Vec<u8>> {
    let mut bytes = std::fs::read(path).ok()?;

    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }

    if bytes.is_empty() { None } else { Some(bytes) }
}

#[cfg(unix)]
fn path_is_dir(bytes: &[u8]) -> bool {
    Path::new(OsStr::from_bytes(bytes)).is_dir()
}

#[cfg(not(unix))]
fn path_is_dir(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(|path| Path::new(path).is_dir())
        .unwrap_or(false)
}

#[cfg(unix)]
fn path_is_file(bytes: &[u8]) -> bool {
    Path::new(OsStr::from_bytes(bytes)).is_file()
}

#[cfg(not(unix))]
fn path_is_file(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(|path| Path::new(path).is_file())
        .unwrap_or(false)
}

#[cfg(unix)]
fn path_is_link(bytes: &[u8]) -> bool {
    std::fs::symlink_metadata(Path::new(OsStr::from_bytes(bytes)))
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn path_is_link(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::symlink_metadata(Path::new(path)).ok())
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(unix)]
fn path_is_readable(bytes: &[u8]) -> bool {
    let path = Path::new(OsStr::from_bytes(bytes));
    if path.is_dir() {
        return std::fs::read_dir(path).is_ok();
    }
    std::fs::File::open(path).is_ok()
}

#[cfg(not(unix))]
fn path_is_readable(bytes: &[u8]) -> bool {
    let Ok(path) = std::str::from_utf8(bytes) else {
        return false;
    };
    let path = Path::new(path);
    if path.is_dir() {
        return std::fs::read_dir(path).is_ok();
    }
    std::fs::File::open(path).is_ok()
}

#[cfg(unix)]
fn path_is_writable(bytes: &[u8]) -> bool {
    let path = Path::new(OsStr::from_bytes(bytes));
    path_is_writable_path(path)
}

#[cfg(not(unix))]
fn path_is_writable(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(Path::new)
        .map(path_is_writable_path)
        .unwrap_or(false)
}

fn path_is_writable_path(path: &Path) -> bool {
    if path.is_dir() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let probe = path.join(format!(".echo_writable_probe_{nanos}"));
        return OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
            .map(|_| {
                let _ = std::fs::remove_file(&probe);
                true
            })
            .unwrap_or(false);
    }

    OpenOptions::new().append(true).open(path).is_ok()
}

#[cfg(unix)]
fn path_is_executable(bytes: &[u8]) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn path_is_executable(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .ok()
        .map(Path::new)
        .filter(|path| path.is_file())
        .and_then(|path| path.extension())
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "exe" | "bat" | "cmd" | "com"
            )
        })
        .unwrap_or(false)
}

#[cfg(unix)]
fn path_filesize(bytes: &[u8]) -> Option<u64> {
    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.len())
}

#[cfg(not(unix))]
fn path_filesize(bytes: &[u8]) -> Option<u64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .map(|metadata| metadata.len())
}

#[cfg(unix)]
fn path_fileatime(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.atime())
}

#[cfg(not(unix))]
fn path_fileatime(bytes: &[u8]) -> Option<i64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .and_then(|metadata| metadata.accessed().ok())
        .and_then(system_time_unix_timestamp)
}

#[cfg(unix)]
fn path_filectime(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.ctime())
}

#[cfg(not(unix))]
fn path_filectime(bytes: &[u8]) -> Option<i64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .and_then(|metadata| metadata.created().ok())
        .and_then(system_time_unix_timestamp)
}

#[cfg(unix)]
fn path_filemtime(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.mtime())
}

#[cfg(not(unix))]
fn path_filemtime(bytes: &[u8]) -> Option<i64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .and_then(|metadata| metadata.modified().ok())
        .and_then(system_time_unix_timestamp)
}

#[cfg(unix)]
fn path_fileinode(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .and_then(|metadata| i64::try_from(metadata.ino()).ok())
}

#[cfg(not(unix))]
fn path_fileinode(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_fileowner(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.uid() as i64)
}

#[cfg(not(unix))]
fn path_fileowner(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_filegroup(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.gid() as i64)
}

#[cfg(not(unix))]
fn path_filegroup(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_fileperms(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.mode() as i64)
}

#[cfg(not(unix))]
fn path_fileperms(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_filetype(bytes: &[u8]) -> Option<Vec<u8>> {
    use std::os::unix::fs::FileTypeExt;

    let file_type = std::fs::symlink_metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()?
        .file_type();
    let name = if file_type.is_symlink() {
        "link"
    } else if file_type.is_dir() {
        "dir"
    } else if file_type.is_file() {
        "file"
    } else if file_type.is_fifo() {
        "fifo"
    } else if file_type.is_char_device() {
        "char"
    } else if file_type.is_block_device() {
        "block"
    } else if file_type.is_socket() {
        "socket"
    } else {
        "unknown"
    };
    Some(name.as_bytes().to_vec())
}

#[cfg(not(unix))]
fn path_filetype(bytes: &[u8]) -> Option<Vec<u8>> {
    let path = std::str::from_utf8(bytes).ok()?;
    let file_type = std::fs::symlink_metadata(Path::new(path)).ok()?.file_type();
    let name = if file_type.is_symlink() {
        "link"
    } else if file_type.is_dir() {
        "dir"
    } else if file_type.is_file() {
        "file"
    } else {
        "unknown"
    };
    Some(name.as_bytes().to_vec())
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

const PHP_FILE_APPEND: i64 = 8;

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

#[cfg(unix)]
fn path_readlink(bytes: &[u8]) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    std::fs::read_link(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|path| path.into_os_string().into_vec())
}

#[cfg(not(unix))]
fn path_readlink(bytes: &[u8]) -> Option<Vec<u8>> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::read_link(Path::new(path)).ok())
        .and_then(|path| path.into_os_string().into_string().ok())
        .map(String::into_bytes)
}

fn path_link(target: &[u8], link: &[u8]) -> bool {
    match (path_buf_from_bytes(target), path_buf_from_bytes(link)) {
        (Some(target), Some(link)) => std::fs::hard_link(target, link).is_ok(),
        _ => false,
    }
}

#[cfg(unix)]
fn path_symlink(target: &[u8], link: &[u8]) -> bool {
    std::os::unix::fs::symlink(OsStr::from_bytes(target), OsStr::from_bytes(link)).is_ok()
}

#[cfg(windows)]
fn path_symlink(target: &[u8], link: &[u8]) -> bool {
    match (path_buf_from_bytes(target), path_buf_from_bytes(link)) {
        (Some(target), Some(link)) => {
            if target.is_dir() {
                std::os::windows::fs::symlink_dir(target, link).is_ok()
            } else {
                std::os::windows::fs::symlink_file(target, link).is_ok()
            }
        }
        _ => false,
    }
}

#[cfg(all(not(unix), not(windows)))]
fn path_symlink(_target: &[u8], _link: &[u8]) -> bool {
    false
}

#[cfg(unix)]
fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    Some(path.into_os_string().into_vec())
}

#[cfg(not(unix))]
fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    path.into_os_string()
        .into_string()
        .ok()
        .map(String::into_bytes)
}

fn path_tempnam(directory: &[u8], prefix: &[u8]) -> Option<Vec<u8>> {
    let requested = path_buf_from_bytes(directory)?;
    let fallback = env::temp_dir();
    create_temp_file_in(&requested, prefix).or_else(|| create_temp_file_in(&fallback, prefix))
}

fn create_temp_file_in(directory: &Path, prefix: &[u8]) -> Option<Vec<u8>> {
    if !directory.is_dir() {
        return None;
    }

    let prefix = &prefix[..prefix.len().min(63)];
    for _ in 0..128 {
        let mut name = Vec::with_capacity(prefix.len() + 16);
        name.extend_from_slice(prefix);
        let unique = php_uniqid(b"", false);
        name.extend_from_slice(&unique);

        let mut path = directory.to_path_buf();
        push_path_component_from_bytes(&mut path, &name)?;
        if create_temp_file(&path) {
            return path_bytes(path);
        }
    }

    None
}

#[cfg(unix)]
fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    path.push(OsStr::from_bytes(component));
    Some(())
}

#[cfg(not(unix))]
fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    path.push(std::str::from_utf8(component).ok()?);
    Some(())
}

fn create_temp_file(path: &Path) -> bool {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    configure_temp_file_mode(&mut options);
    options.open(path).is_ok()
}

#[cfg(unix)]
fn configure_temp_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.mode(0o600);
}

#[cfg(not(unix))]
fn configure_temp_file_mode(_options: &mut OpenOptions) {}

fn php_uniqid(prefix: &[u8], more_entropy: bool) -> Vec<u8> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = duration.as_secs() as u32;
    let micros = duration.subsec_micros();
    let counter = NEXT_UNIQID_COUNTER.fetch_add(1, Ordering::Relaxed) as u32;
    let micros = (micros.wrapping_add(counter)) % 0x100000;

    let mut id = prefix.to_vec();
    id.extend_from_slice(format!("{seconds:08x}{micros:05x}").as_bytes());
    if more_entropy {
        let entropy = ((duration.subsec_nanos() as u64)
            ^ ((counter as u64).wrapping_mul(1_103_515_245)))
            % 1_000_000_000;
        id.extend_from_slice(format!(".{entropy:09}").as_bytes());
    }
    id
}

#[cfg(unix)]
fn path_buf_from_bytes(bytes: &[u8]) -> Option<PathBuf> {
    Some(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
fn path_buf_from_bytes(bytes: &[u8]) -> Option<PathBuf> {
    std::str::from_utf8(bytes).ok().map(PathBuf::from)
}

fn path_touch(bytes: &[u8], mtime: Option<i64>, atime: Option<i64>) -> bool {
    let Some(path) = path_buf_from_bytes(bytes) else {
        return false;
    };

    if OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .is_err()
    {
        return false;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .unwrap_or(0);
    let modified = mtime.unwrap_or(now);
    let accessed = atime.unwrap_or(modified);

    filetime::set_file_times(
        &path,
        FileTime::from_unix_time(accessed, 0),
        FileTime::from_unix_time(modified, 0),
    )
    .is_ok()
}

fn path_copy(from: &[u8], to: &[u8]) -> bool {
    match (path_buf_from_bytes(from), path_buf_from_bytes(to)) {
        (Some(from), Some(to)) => std::fs::copy(from, to).is_ok(),
        _ => false,
    }
}

fn path_rename(from: &[u8], to: &[u8]) -> bool {
    match (path_buf_from_bytes(from), path_buf_from_bytes(to)) {
        (Some(from), Some(to)) => std::fs::rename(from, to).is_ok(),
        _ => false,
    }
}

fn path_unlink(bytes: &[u8]) -> bool {
    path_buf_from_bytes(bytes)
        .map(std::fs::remove_file)
        .is_some_and(|result| result.is_ok())
}

fn path_mkdir(bytes: &[u8], permissions: i64, recursive: bool) -> bool {
    let Some(path) = path_buf_from_bytes(bytes) else {
        return false;
    };
    if path.exists() {
        return false;
    }

    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(recursive);
    configure_dir_builder_mode(&mut builder, permissions);
    builder.create(path).is_ok()
}

#[cfg(unix)]
fn configure_dir_builder_mode(builder: &mut std::fs::DirBuilder, permissions: i64) {
    use std::os::unix::fs::DirBuilderExt;

    builder.mode(permissions as u32);
}

#[cfg(not(unix))]
fn configure_dir_builder_mode(_builder: &mut std::fs::DirBuilder, _permissions: i64) {}

fn path_rmdir(bytes: &[u8]) -> bool {
    path_buf_from_bytes(bytes)
        .map(std::fs::remove_dir)
        .is_some_and(|result| result.is_ok())
}

#[cfg(not(unix))]
fn system_time_unix_timestamp(time: SystemTime) -> Option<i64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
}

#[cfg(unix)]
fn path_realpath(bytes: &[u8]) -> Option<Vec<u8>> {
    let path = if bytes.is_empty() {
        Path::new(".")
    } else {
        Path::new(OsStr::from_bytes(bytes))
    };
    std::fs::canonicalize(path)
        .ok()
        .map(|path| path.as_os_str().as_bytes().to_vec())
}

#[cfg(not(unix))]
fn path_realpath(bytes: &[u8]) -> Option<Vec<u8>> {
    let path = if bytes.is_empty() {
        "."
    } else {
        std::str::from_utf8(bytes).ok()?
    };
    std::fs::canonicalize(path)
        .ok()
        .and_then(|path| path.into_os_string().into_string().ok())
        .map(String::into_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_basename(path: EchoValue, suffix: EchoValue) -> EchoValue {
    let Some(path) = path.string_bytes() else {
        return EchoValue::error();
    };
    let Some(suffix) = suffix.string_bytes() else {
        return EchoValue::error();
    };

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(php_basename(
        &path, &suffix,
    )))))
}

fn php_basename(path: &[u8], suffix: &[u8]) -> Vec<u8> {
    let trimmed_end = path
        .iter()
        .rposition(|byte| *byte != b'/')
        .map_or(0, |position| position + 1);
    let path = &path[..trimmed_end];
    let start = path
        .iter()
        .rposition(|byte| *byte == b'/')
        .map_or(0, |position| position + 1);
    let mut basename = path[start..].to_vec();

    if !suffix.is_empty() && basename.ends_with(suffix) {
        basename.truncate(basename.len() - suffix.len());
    }

    basename
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_dirname(path: EchoValue, levels: EchoValue) -> EchoValue {
    let Some(path) = path.string_bytes() else {
        return EchoValue::error();
    };
    let Some(levels) = levels.php_int_value() else {
        return EchoValue::error();
    };
    if levels <= 0 {
        return EchoValue::error();
    }

    let mut dirname = path;
    for _ in 0..levels {
        dirname = php_dirname_once(&dirname);
    }

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(dirname))))
}

fn php_dirname_once(path: &[u8]) -> Vec<u8> {
    let Some(last_non_slash) = path.iter().rposition(|byte| *byte != b'/') else {
        return b"/".to_vec();
    };
    let path = &path[..=last_non_slash];
    let Some(last_slash) = path.iter().rposition(|byte| *byte == b'/') else {
        return b".".to_vec();
    };
    if last_slash == 0 {
        return b"/".to_vec();
    }

    let parent = &path[..last_slash];
    let parent_end = parent
        .iter()
        .rposition(|byte| *byte != b'/')
        .map_or(0, |position| position + 1);
    if parent_end == 0 {
        b"/".to_vec()
    } else {
        parent[..parent_end].to_vec()
    }
}

fn decode_base64_non_strict(bytes: &[u8]) -> Vec<u8> {
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hex2bin(value: EchoValue) -> EchoValue {
    match value.string_bytes().and_then(|bytes| decode_hex(&bytes)) {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes)))),
        None => EchoValue::bool(false),
    }
}

fn decode_hex(bytes: &[u8]) -> Option<Vec<u8>> {
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

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_trim(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(trim_bytes(
            &bytes, true, true,
        ))))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ltrim(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(trim_bytes(
            &bytes, true, false,
        ))))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rtrim(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(trim_bytes(
            &bytes, false, true,
        ))))),
        None => EchoValue::error(),
    }
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
enum PhpNumber {
    Int(i64),
    Float(f64),
}

impl PhpNumber {
    fn coerce(value: EchoValue) -> Option<Self> {
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

    const fn as_float(self) -> f64 {
        match self {
            Self::Int(value) => value as f64,
            Self::Float(value) => value,
        }
    }

    fn into_echo_value(self) -> EchoValue {
        match self {
            Self::Int(value) => EchoValue::int(value),
            Self::Float(value) => EchoValue::float(value),
        }
    }
}

fn php_number_add(left: PhpNumber, right: PhpNumber) -> PhpNumber {
    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) => left
            .checked_add(right)
            .map(PhpNumber::Int)
            .unwrap_or_else(|| PhpNumber::Float(left as f64 + right as f64)),
        _ => PhpNumber::Float(left.as_float() + right.as_float()),
    }
}

fn php_number_mul(left: PhpNumber, right: PhpNumber) -> PhpNumber {
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

fn php_array_union(left: EchoValue, right: EchoValue) -> EchoValue {
    if !left.is_array() || !right.is_array() {
        return EchoValue::error();
    }

    let Some(left) = (unsafe { (left.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };
    let Some(right) = (unsafe { (right.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut keys = left.keys.clone();
    let mut values = left.values.clone();
    for (key, value) in right.keys.iter().zip(&right.values) {
        if !keys.contains(key) {
            keys.push(key.clone());
            values.push(*value);
        }
    }
    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
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

fn next_array_append_key(array: &EchoArray) -> EchoArrayKey {
    let next = array
        .keys
        .iter()
        .filter_map(|key| match key {
            EchoArrayKey::Int(value) => Some(*value),
            EchoArrayKey::String(_) => None,
        })
        .filter(|value| *value >= 0)
        .max()
        .map(|value| value.saturating_add(1))
        .unwrap_or(0);
    EchoArrayKey::Int(next)
}

fn parse_php_array_integer_key(bytes: &[u8]) -> Option<i64> {
    let text = std::str::from_utf8(bytes).ok()?;
    if text == "0" {
        return Some(0);
    }
    if let Some(rest) = text.strip_prefix('-') {
        if rest.starts_with('0') || rest.is_empty() {
            return None;
        }
    } else if text.starts_with('0') || text.is_empty() {
        return None;
    }
    text.parse::<i64>().ok()
}

fn format_php_float(value: f64) -> String {
    if value.is_nan() {
        return "NAN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-INF".to_string()
        } else {
            "INF".to_string()
        };
    }

    let formatted = format!("{value:.14}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
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

fn echo_math_pow(base: f64, exponent: f64) -> f64 {
    if exponent == 0.0 {
        return 1.0;
    }
    if base == 0.0 {
        return if exponent.is_sign_negative() {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if base < 0.0 {
        return f64::NAN;
    }
    if base.is_nan() || exponent.is_nan() {
        return f64::NAN;
    }
    if base.is_infinite() {
        return if exponent.is_sign_negative() {
            0.0
        } else {
            f64::INFINITY
        };
    }

    echo_math_exp(echo_math_ln(base) * exponent)
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

fn php_float_coercion(value: EchoValue) -> Option<f64> {
    match value.kind {
        ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(0.0),
        ECHO_VALUE_BOOL => Some(if value.payload == 0 { 0.0 } else { 1.0 }),
        ECHO_VALUE_INT => Some(value.payload as i64 as f64),
        ECHO_VALUE_FLOAT => Some(f64::from_bits(value.payload)),
        ECHO_VALUE_STRING => unsafe {
            let bytes = &(value.payload as *const EchoString).as_ref()?.bytes;
            let text = std::str::from_utf8(trim_ascii(bytes)).ok()?;
            if text.is_empty() {
                return None;
            }
            text.parse::<f64>().ok()
        },
        _ => None,
    }
}

fn php_float_cast(value: EchoValue) -> Option<f64> {
    match value.kind {
        ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(0.0),
        ECHO_VALUE_BOOL => Some(if value.payload == 0 { 0.0 } else { 1.0 }),
        ECHO_VALUE_INT => Some(value.payload as i64 as f64),
        ECHO_VALUE_FLOAT => Some(f64::from_bits(value.payload)),
        ECHO_VALUE_STRING => unsafe {
            let bytes = &(value.payload as *const EchoString).as_ref()?.bytes;
            Some(parse_php_decimal_float_prefix(bytes))
        },
        _ => None,
    }
}

fn parse_php_decimal_float_prefix(bytes: &[u8]) -> f64 {
    let bytes = trim_ascii_start(bytes);
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
        return 0.0;
    }

    let mut end = index;
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_start = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'-' | b'+')) {
            index += 1;
        }
        if consume_ascii_digits(bytes, &mut index) > 0 {
            end = index;
        } else {
            end = exponent_start;
        }
    }

    std::str::from_utf8(&bytes[..end])
        .ok()
        .and_then(|text| text.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn consume_ascii_digits(bytes: &[u8], index: &mut usize) -> usize {
    let start = *index;
    while bytes.get(*index).is_some_and(u8::is_ascii_digit) {
        *index += 1;
    }
    *index - start
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let bytes = trim_ascii_start(bytes);
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(0, |index| index + 1);
    &bytes[..end]
}

fn trim_ascii_start(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    &bytes[start..]
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_addslashes(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => {
            let mut escaped = Vec::with_capacity(bytes.len());
            for byte in bytes {
                match byte {
                    b'\'' | b'"' | b'\\' => {
                        escaped.push(b'\\');
                        escaped.push(byte);
                    }
                    b'\0' => escaped.extend_from_slice(b"\\0"),
                    other => escaped.push(other),
                }
            }
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(escaped))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stripslashes(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => {
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

            EchoValue::string(Box::into_raw(Box::new(EchoString::new(stripped))))
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quoted_printable_encode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            quoted_printable_encode_bytes(&bytes),
        )))),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quoted_printable_decode(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            quoted_printable_decode_bytes(&bytes),
        )))),
        None => EchoValue::error(),
    }
}

fn quoted_printable_encode_bytes(bytes: &[u8]) -> Vec<u8> {
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

fn quoted_printable_decode_bytes(bytes: &[u8]) -> Vec<u8> {
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_nl2br(value: EchoValue, use_xhtml: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let use_xhtml = use_xhtml.bool_value().unwrap_or(true);
    let marker: &[u8] = if use_xhtml { b"<br />" } else { b"<br>" };

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(php_nl2br(
        &bytes, marker,
    )))))
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
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(quoted))))
        }
        None => EchoValue::error(),
    }
}

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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(replace_bytes(
        &subject, &search, &replace, false,
    )))))
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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(replace_bytes(
        &subject, &search, &replace, true,
    )))))
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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(php_strtr(
        &value, &from, &to,
    )))))
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
        return EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))));
    };
    if pad_string.is_empty() {
        return EchoValue::error();
    }

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(php_str_pad(
        &bytes,
        length,
        &pad_string,
        pad_type,
    )))))
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
pub extern "C" fn echo_php_str_split(value: EchoValue, length: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::error();
    };
    if length == 0 {
        return EchoValue::error();
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        bytes
            .chunks(length)
            .map(|chunk| {
                EchoValue::string(Box::into_raw(Box::new(EchoString::new(chunk.to_vec()))))
            })
            .collect(),
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_chunk_split(
    value: EchoValue,
    length: EchoValue,
    separator: EchoValue,
) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    let Some(separator) = separator.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::error();
    };
    if length == 0 {
        return EchoValue::error();
    }

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(php_chunk_split(
        &bytes, length, &separator,
    )))))
}

fn php_chunk_split(bytes: &[u8], length: usize, separator: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len() + separator.len());
    for chunk in bytes.chunks(length) {
        result.extend_from_slice(chunk);
        result.extend_from_slice(separator);
    }
    if bytes.is_empty() {
        result.extend_from_slice(separator);
    }
    result
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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        bytes[start as usize..].to_vec(),
    ))))
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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        haystack[position..].to_vec(),
    ))))
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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        haystack[position..].to_vec(),
    ))))
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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        haystack[position..].to_vec(),
    ))))
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

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        value[position..].to_vec(),
    ))))
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_call_function(ptr: *const u8, len: usize) -> EchoValue {
    if ptr.is_null() && len != 0 {
        return EchoValue::error();
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let Ok(name) = std::str::from_utf8(bytes) else {
        return EchoValue::error();
    };

    let callable = EchoCallable::Function(EchoSymbol::new(name));
    echo_call(&callable, &[]).unwrap_or_else(|_| EchoValue::error())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_start() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start());
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_start_value(callback: EchoValue) -> bool {
    let Ok(callback) = echo_normalize_callable(callback) else {
        return false;
    };

    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start_with_callback(callback));
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_clean() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_clean())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_flush() -> bool {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let ok = runtime.borrow_mut().ob_flush(&mut stdout);
        write_stdout(&stdout);
        ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_end_flush() -> bool {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let ok = runtime.borrow_mut().ob_end_flush(&mut stdout);
        write_stdout(&stdout);
        ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_end_clean() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_end_clean())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_clean() -> EchoValue {
    OUTPUT.with(|runtime| match runtime.borrow_mut().ob_get_clean() {
        Some(value) => EchoValue::string(Box::into_raw(Box::new(value))),
        None => EchoValue::bool(false),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_contents() -> EchoValue {
    OUTPUT.with(|runtime| match runtime.borrow().ob_get_contents() {
        Some(value) => EchoValue::string(Box::into_raw(Box::new(value))),
        None => EchoValue::bool(false),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_flush() -> EchoValue {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let value = runtime.borrow_mut().ob_get_flush(&mut stdout);
        write_stdout(&stdout);
        match value {
            Some(value) => EchoValue::string(Box::into_raw(Box::new(value))),
            None => EchoValue::bool(false),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_level() -> EchoValue {
    // PHP `ob_get_level()` returns zero when inactive; the first active buffer is level 1.
    // Source: https://www.php.net/manual/en/function.ob-get-level.php
    OUTPUT.with(|runtime| EchoValue::int(runtime.borrow().level() as i64))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_length() -> EchoValue {
    OUTPUT.with(|runtime| match runtime.borrow().ob_get_length() {
        Some(len) => EchoValue::int(len as i64),
        None => EchoValue::bool(false),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_shutdown() {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        runtime.borrow_mut().shutdown(&mut stdout);
        write_stdout(&stdout);
    });

    if ASSERT_FAILURES.load(Ordering::Relaxed) > 0 {
        std::process::exit(1);
    }
}

fn write_stdout(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }

    if EXECUTION.with(|execution| {
        let mut execution = execution.borrow_mut();
        if let Some(stdout) = execution.stdout.as_mut() {
            stdout.extend_from_slice(bytes);
            true
        } else {
            false
        }
    }) {
        return;
    }

    let mut stdout = std_io::stdout().lock();
    stdout
        .write_all(bytes)
        .expect("failed to write Echo runtime output");
    stdout.flush().expect("failed to flush Echo runtime output");
}

fn write_runtime_output(bytes: &[u8]) {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        runtime.borrow_mut().write(bytes, &mut stdout);
        write_stdout(&stdout);
    });
}

fn record_assertion(passed: bool, message: &str) {
    if passed {
        return;
    }

    ASSERT_FAILURES.fetch_add(1, Ordering::Relaxed);
    eprintln!("{message}");
}

fn echo_values_equal(left: EchoValue, right: EchoValue) -> bool {
    if left.kind != right.kind {
        return false;
    }

    match left.kind {
        ECHO_VALUE_NULL => true,
        ECHO_VALUE_BOOL | ECHO_VALUE_INT | ECHO_VALUE_FLOAT => left.payload == right.payload,
        ECHO_VALUE_STRING => left.string_bytes() == right.string_bytes(),
        ECHO_VALUE_ARRAY => echo_arrays_equal(left, right),
        ECHO_VALUE_LIST => echo_lists_equal(left, right),
        _ => left.payload == right.payload,
    }
}

fn php_values_equal(left: EchoValue, right: EchoValue) -> bool {
    if let (Some(left), Some(right)) = (PhpNumber::coerce(left), PhpNumber::coerce(right)) {
        return left.as_float() == right.as_float();
    }

    match (left.string_bytes(), right.string_bytes()) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

fn echo_arrays_equal(left: EchoValue, right: EchoValue) -> bool {
    let Some(left) = (unsafe { (left.payload as *const EchoArray).as_ref() }) else {
        return false;
    };
    let Some(right) = (unsafe { (right.payload as *const EchoArray).as_ref() }) else {
        return false;
    };

    left.values.len() == right.values.len()
        && left
            .values
            .iter()
            .zip(&right.values)
            .all(|(left, right)| echo_values_equal(*left, *right))
}

fn echo_lists_equal(left: EchoValue, right: EchoValue) -> bool {
    let Some(left) = (unsafe { (left.payload as *const EchoList).as_ref() }) else {
        return false;
    };
    let Some(right) = (unsafe { (right.payload as *const EchoList).as_ref() }) else {
        return false;
    };

    left.values.len() == right.values.len()
        && left
            .values
            .iter()
            .zip(&right.values)
            .all(|(left, right)| echo_values_equal(*left, *right))
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
    fn writes_to_stdout_without_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.write(b"hello", &mut stdout);

        assert_eq!(stdout, b"hello");
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
    fn end_flush_writes_buffer_to_stdout() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"buffered", &mut stdout);
        assert!(stdout.is_empty());

        assert!(runtime.ob_end_flush(&mut stdout));

        assert_eq!(stdout, b"buffered");
        assert_eq!(runtime.level(), 0);
    }

    #[test]
    fn flush_clears_buffer_but_keeps_it_active() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"x", &mut stdout);
        assert!(runtime.ob_flush(&mut stdout));
        runtime.write(b"y", &mut stdout);
        assert!(runtime.ob_end_flush(&mut stdout));

        assert_eq!(stdout, b"xy");
    }

    #[test]
    fn flush_writes_to_stdout_without_ending_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"x", &mut stdout);
        assert!(runtime.ob_flush(&mut stdout));

        assert_eq!(stdout, b"x");
        assert_eq!(runtime.level(), 1);
    }

    #[test]
    fn clean_discards_buffer_but_keeps_it_active() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"discarded", &mut stdout);
        assert!(runtime.ob_clean());
        runtime.write(b"kept", &mut stdout);
        assert!(runtime.ob_end_flush(&mut stdout));

        assert_eq!(stdout, b"kept");
    }

    #[test]
    fn end_clean_discards_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"discarded", &mut stdout);
        assert!(runtime.ob_end_clean());
        runtime.write(b"kept", &mut stdout);

        assert_eq!(stdout, b"kept");
    }

    #[test]
    fn nested_end_flush_writes_to_parent_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"A", &mut stdout);
        runtime.ob_start();
        runtime.write(b"B", &mut stdout);
        assert!(runtime.ob_end_flush(&mut stdout));
        runtime.write(b"C", &mut stdout);
        assert!(stdout.is_empty());

        assert!(runtime.ob_end_flush(&mut stdout));

        assert_eq!(stdout, b"ABC");
    }

    #[test]
    fn nested_flush_writes_to_parent_buffer_and_keeps_inner_active() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"A", &mut stdout);
        runtime.ob_start();
        runtime.write(b"B", &mut stdout);
        assert!(runtime.ob_flush(&mut stdout));
        runtime.write(b"C", &mut stdout);
        assert!(runtime.ob_end_flush(&mut stdout));
        runtime.write(b"D", &mut stdout);
        assert!(stdout.is_empty());

        assert!(runtime.ob_end_flush(&mut stdout));

        assert_eq!(stdout, b"ABCD");
    }

    #[test]
    fn shutdown_flushes_open_buffers_inside_out() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"A", &mut stdout);
        runtime.ob_start();
        runtime.write(b"B", &mut stdout);

        runtime.shutdown(&mut stdout);

        assert_eq!(stdout, b"AB");
        assert_eq!(runtime.level(), 0);
    }

    #[test]
    fn get_contents_returns_copy_without_cleaning_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"A", &mut stdout);
        let value = runtime.ob_get_contents().expect("active buffer");
        runtime.write(b"B", &mut stdout);
        assert!(runtime.ob_end_clean());

        assert_eq!(value.bytes, b"A");
        assert!(stdout.is_empty());
    }

    #[test]
    fn get_clean_returns_buffer_and_turns_it_off() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"buffered", &mut stdout);
        let value = runtime.ob_get_clean().expect("active buffer");
        runtime.write(b"after", &mut stdout);

        assert_eq!(value.bytes, b"buffered");
        assert_eq!(runtime.level(), 0);
        assert_eq!(stdout, b"after");
    }

    #[test]
    fn get_flush_returns_and_flushes_buffer_then_turns_it_off() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"buffered", &mut stdout);
        let value = runtime.ob_get_flush(&mut stdout).expect("active buffer");
        runtime.write(b"after", &mut stdout);

        assert_eq!(value.bytes, b"buffered");
        assert_eq!(runtime.level(), 0);
        assert_eq!(stdout, b"bufferedafter");
    }

    #[test]
    fn nested_get_flush_writes_to_parent_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"outer:", &mut stdout);
        runtime.ob_start();
        runtime.write(b"inner", &mut stdout);

        let value = runtime.ob_get_flush(&mut stdout).expect("active buffer");
        runtime.write(b"|after:", &mut stdout);
        runtime.write(&value.bytes, &mut stdout);
        assert!(stdout.is_empty());

        assert!(runtime.ob_end_flush(&mut stdout));

        assert_eq!(value.bytes, b"inner");
        assert_eq!(stdout, b"outer:inner|after:inner");
    }

    #[test]
    fn nested_get_clean_does_not_write_to_parent_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.ob_start();
        runtime.write(b"outer:", &mut stdout);
        runtime.ob_start();
        runtime.write(b"inner", &mut stdout);

        let value = runtime.ob_get_clean().expect("active buffer");
        runtime.write(b"|after:", &mut stdout);
        runtime.write(&value.bytes, &mut stdout);
        assert!(stdout.is_empty());

        assert!(runtime.ob_end_flush(&mut stdout));

        assert_eq!(value.bytes, b"inner");
        assert_eq!(stdout, b"outer:|after:inner");
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
    fn float_scalar_math_builtins_preserve_php_scalar_behavior() {
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

        unsafe {
            drop(Box::from_raw(prefixed));
            drop(Box::from_raw(invalid));
            drop(Box::from_raw(offset));
            drop(Box::from_raw(exponent));
        }
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
    fn chr_and_bin2hex_preserve_php_byte_behavior() {
        let numeric = Box::into_raw(Box::new(EchoString {
            bytes: "321".as_bytes().to_vec(),
        }));
        let text = Box::into_raw(Box::new(EchoString {
            bytes: "Echo".as_bytes().to_vec(),
        }));
        let non_ascii = Box::into_raw(Box::new(EchoString {
            bytes: "Ä".as_bytes().to_vec(),
        }));

        assert_eq!(
            echo_php_chr(EchoValue::int(65)).string_bytes(),
            Some("A".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_chr(EchoValue::string(numeric)).string_bytes(),
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
        assert_eq!(
            echo_php_bin2hex(EchoValue::string(text)).string_bytes(),
            Some("4563686f".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_bin2hex(EchoValue::string(non_ascii)).string_bytes(),
            Some("c384".as_bytes().to_vec())
        );
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
        assert_eq!(
            echo_php_escapeshellarg(EchoValue::string(text)).string_bytes(),
            Some("'Echo'".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(text));
            drop(Box::from_raw(non_ascii));
        }
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
    fn hex2bin_and_str_repeat_preserve_php_byte_behavior() {
        let hex = Box::into_raw(Box::new(EchoString {
            bytes: "c384".as_bytes().to_vec(),
        }));
        let upper_hex = Box::into_raw(Box::new(EchoString {
            bytes: "4563686F".as_bytes().to_vec(),
        }));
        let invalid_hex = Box::into_raw(Box::new(EchoString {
            bytes: "f".as_bytes().to_vec(),
        }));
        let repeated = Box::into_raw(Box::new(EchoString {
            bytes: "xo".as_bytes().to_vec(),
        }));
        let empty_repeat = Box::into_raw(Box::new(EchoString {
            bytes: "x".as_bytes().to_vec(),
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
        assert_eq!(
            echo_php_str_repeat(EchoValue::string(repeated), EchoValue::int(3)).string_bytes(),
            Some("xoxoxo".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_str_repeat(EchoValue::string(empty_repeat), EchoValue::int(0)).string_bytes(),
            Some(Vec::new())
        );

        unsafe {
            drop(Box::from_raw(hex));
            drop(Box::from_raw(upper_hex));
            drop(Box::from_raw(invalid_hex));
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

    #[test]
    fn ob_start_with_callback_stores_callback_frame() {
        let mut runtime = OutputRuntime::new();
        let callback = EchoCallable::Function(EchoSymbol::new("filter"));

        runtime.ob_start_with_callback(Some(callback.clone()));

        assert_eq!(runtime.level(), 1);
        assert_eq!(runtime.stack[0].callback, Some(callback));
    }

    #[test]
    fn get_length_returns_active_buffer_byte_length() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        assert_eq!(runtime.ob_get_length(), None);

        runtime.ob_start();
        runtime.write(b"abc", &mut stdout);

        assert_eq!(runtime.ob_get_length(), Some(3));
        assert!(stdout.is_empty());
    }
}
