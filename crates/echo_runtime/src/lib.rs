pub mod net;
pub mod poll;
pub mod sched;
pub mod task;

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct OutputRuntime {
    stack: Vec<OutputBuffer>,
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

static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);

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
            ECHO_VALUE_TASK => Some(b"Object".to_vec()),
            _ => None,
        }
    }

    pub const fn is_string(self) -> bool {
        self.kind == ECHO_VALUE_STRING
    }

    pub const fn is_array(self) -> bool {
        self.kind == ECHO_VALUE_ARRAY
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
        _ => {}
    }
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
pub extern "C" fn echo_php_strlen(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::int(bytes.len() as i64),
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
