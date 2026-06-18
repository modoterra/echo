pub mod net;
pub mod poll;
pub mod sched;
pub mod task;

use std::cell::RefCell;
use std::cmp::Ordering as CmpOrdering;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

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
    values: Vec<EchoValue>,
}

impl EchoArray {
    fn new() -> Self {
        Self { values: Vec::new() }
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

static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);
static ASSERT_FAILURES: AtomicUsize = AtomicUsize::new(0);

const REFLECTION_SOURCE_PHP_BUILTIN: i32 = 1;

const PHP_DEFAULT_TRIM_BYTES: &[u8] = b" \n\r\t\x0b\0";

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
            ECHO_VALUE_STRING => unsafe {
                (self.payload as *const EchoString)
                    .as_ref()
                    .map(|value| value.bytes.clone())
            },
            ECHO_VALUE_ARRAY => Some(b"Array".to_vec()),
            ECHO_VALUE_LIST => Some(b"List".to_vec()),
            ECHO_VALUE_TASK | ECHO_VALUE_OBJECT => Some(b"Object".to_vec()),
            _ => None,
        }
    }

    fn int_value(self) -> Option<i64> {
        match self.kind {
            ECHO_VALUE_BOOL => Some(if self.payload == 0 { 0 } else { 1 }),
            ECHO_VALUE_INT => Some(self.payload as i64),
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
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(self.payload as *const EchoString).as_ref()?.bytes;
                Some(!bytes.is_empty() && bytes != b"0")
            },
            ECHO_VALUE_ARRAY | ECHO_VALUE_LIST => Some(true),
            ECHO_VALUE_TASK | ECHO_VALUE_TCP_LISTENER | ECHO_VALUE_TCP_CONNECTION => Some(true),
            ECHO_VALUE_PENDING => Some(false),
            _ => None,
        }
    }

    fn php_int_value(self) -> Option<i64> {
        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(0),
            ECHO_VALUE_BOOL => Some(if self.payload == 0 { 0 } else { 1 }),
            ECHO_VALUE_INT => Some(self.payload as i64),
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
    match value.kind {
        ECHO_VALUE_NULL | ECHO_VALUE_ERROR => {}
        ECHO_VALUE_BOOL => {
            if value.payload != 0 {
                unsafe { echo_write(c"1".as_ptr().cast(), 1) };
            }
        }
        ECHO_VALUE_INT => echo_write_i64(value.payload as i64),
        ECHO_VALUE_STRING => unsafe { echo_write_string(value.payload as *const EchoString) },
        ECHO_VALUE_ARRAY => unsafe { echo_write(c"Array".as_ptr().cast(), 5) },
        ECHO_VALUE_LIST => unsafe { echo_write(c"List".as_ptr().cast(), 4) },
        _ => {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_flush() {
    let _ = io::stdout().flush();
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
            .map_err(|_| io::Error::other("failed to schedule Echo task"))
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
            .map_err(|_| io::Error::other("failed to join Echo task"))
    })
    .unwrap_or_else(|_| EchoValue::error())
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
        ECHO_VALUE_STRING => b"string".as_slice(),
        ECHO_VALUE_ARRAY => b"array".as_slice(),
        ECHO_VALUE_LIST => b"list".as_slice(),
        ECHO_VALUE_TASK => b"task".as_slice(),
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
    if !left.is_int() || !right.is_int() {
        return EchoValue::error();
    }

    EchoValue::int((left.payload as i64) + (right.payload as i64))
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

    values.values.push(value);
    array
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
        ECHO_VALUE_STRING => b"string".as_slice(),
        ECHO_VALUE_ARRAY => b"array".as_slice(),
        ECHO_VALUE_LIST => b"list".as_slice(),
        ECHO_VALUE_TASK | ECHO_VALUE_OBJECT => b"object".as_slice(),
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
pub extern "C" fn echo_php_array_is_list(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array())
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
pub extern "C" fn echo_php_is_float(_value: EchoValue) -> EchoValue {
    EchoValue::bool(false)
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
        ECHO_VALUE_TCP_LISTENER | ECHO_VALUE_TCP_CONNECTION
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
pub extern "C" fn echo_php_bin2hex(value: EchoValue) -> EchoValue {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    match value.string_bytes() {
        Some(bytes) => {
            let mut encoded = Vec::with_capacity(bytes.len() * 2);
            for byte in bytes {
                encoded.push(HEX[(byte >> 4) as usize]);
                encoded.push(HEX[(byte & 0x0f) as usize]);
            }
            EchoValue::string(Box::into_raw(Box::new(EchoString::new(encoded))))
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

    let mut stdout = io::stdout().lock();
    stdout
        .write_all(bytes)
        .expect("failed to write Echo runtime output");
    stdout.flush().expect("failed to flush Echo runtime output");
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
        ECHO_VALUE_BOOL | ECHO_VALUE_INT => left.payload == right.payload,
        ECHO_VALUE_STRING => left.string_bytes() == right.string_bytes(),
        ECHO_VALUE_ARRAY => echo_arrays_equal(left, right),
        ECHO_VALUE_LIST => echo_lists_equal(left, right),
        _ => left.payload == right.payload,
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
            echo_php_bin2hex(EchoValue::string(text)).string_bytes(),
            Some("4563686f".as_bytes().to_vec())
        );
        assert_eq!(
            echo_php_bin2hex(EchoValue::string(non_ascii)).string_bytes(),
            Some("c384".as_bytes().to_vec())
        );

        unsafe {
            drop(Box::from_raw(numeric));
            drop(Box::from_raw(text));
            drop(Box::from_raw(non_ascii));
        }
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
        let array = Box::into_raw(Box::new(EchoArray {
            values: vec![EchoValue::int(1)],
        }));

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
        let array = Box::into_raw(Box::new(EchoArray { values: Vec::new() }));

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
        let array = Box::into_raw(Box::new(EchoArray { values: Vec::new() }));

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
        let array = Box::into_raw(Box::new(EchoArray { values: Vec::new() }));

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
        let array = Box::into_raw(Box::new(EchoArray { values: Vec::new() }));
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
        let array = Box::into_raw(Box::new(EchoArray { values: Vec::new() }));

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
        let array = Box::into_raw(Box::new(EchoArray {
            values: vec![EchoValue::int(1), EchoValue::int(2)],
        }));
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
        let array = Box::into_raw(Box::new(EchoArray { values: Vec::new() }));

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
