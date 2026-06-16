use std::cell::RefCell;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFn {
    EchoWrite,
    EchoWriteI64,
    EchoWriteI64OrFalse,
    EchoWriteString,
    EchoValueString,
    ObStart,
    ObStartValue,
    ObClean,
    ObFlush,
    ObEndFlush,
    ObEndClean,
    ObGetClean,
    ObGetContents,
    ObGetFlush,
    ObGetLength,
    ObGetLevel,
    Shutdown,
}

impl RuntimeFn {
    pub const ALL: &'static [Self] = &[
        Self::EchoWrite,
        Self::EchoWriteI64,
        Self::EchoWriteI64OrFalse,
        Self::EchoWriteString,
        Self::EchoValueString,
        Self::ObStart,
        Self::ObStartValue,
        Self::ObClean,
        Self::ObFlush,
        Self::ObEndFlush,
        Self::ObEndClean,
        Self::ObGetClean,
        Self::ObGetContents,
        Self::ObGetFlush,
        Self::ObGetLength,
        Self::ObGetLevel,
        Self::Shutdown,
    ];

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::EchoWrite => "echo_write",
            Self::EchoWriteI64 => "echo_write_i64",
            Self::EchoWriteI64OrFalse => "echo_write_i64_or_false",
            Self::EchoWriteString => "echo_write_string",
            Self::EchoValueString => "echo_value_string",
            Self::ObStart => "echo_ob_start",
            Self::ObStartValue => "echo_ob_start_value",
            Self::ObClean => "echo_ob_clean",
            Self::ObFlush => "echo_ob_flush",
            Self::ObEndFlush => "echo_ob_end_flush",
            Self::ObEndClean => "echo_ob_end_clean",
            Self::ObGetClean => "echo_ob_get_clean",
            Self::ObGetContents => "echo_ob_get_contents",
            Self::ObGetFlush => "echo_ob_get_flush",
            Self::ObGetLength => "echo_ob_get_length",
            Self::ObGetLevel => "echo_ob_get_level",
            Self::Shutdown => "echo_shutdown",
        }
    }

    pub const fn llvm_decl(self) -> &'static str {
        match self {
            Self::EchoWrite => "declare void @echo_write(ptr, i64)",
            Self::EchoWriteI64 => "declare void @echo_write_i64(i64)",
            Self::EchoWriteI64OrFalse => "declare void @echo_write_i64_or_false(i64)",
            Self::EchoWriteString => "declare void @echo_write_string(ptr)",
            Self::EchoValueString => "declare %EchoValue @echo_value_string(ptr, i64)",
            Self::ObStart => "declare void @echo_ob_start()",
            Self::ObStartValue => "declare i1 @echo_ob_start_value(%EchoValue)",
            Self::ObClean => "declare i1 @echo_ob_clean()",
            Self::ObFlush => "declare i1 @echo_ob_flush()",
            Self::ObEndFlush => "declare i1 @echo_ob_end_flush()",
            Self::ObEndClean => "declare i1 @echo_ob_end_clean()",
            Self::ObGetClean => "declare ptr @echo_ob_get_clean()",
            Self::ObGetContents => "declare ptr @echo_ob_get_contents()",
            Self::ObGetFlush => "declare ptr @echo_ob_get_flush()",
            Self::ObGetLength => "declare i64 @echo_ob_get_length()",
            Self::ObGetLevel => "declare i64 @echo_ob_get_level()",
            Self::Shutdown => "declare void @echo_shutdown()",
        }
    }
}

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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EchoValue {
    pub kind: i32,
    pub payload: u64,
}

const ECHO_VALUE_NULL: i32 = 0;
const ECHO_VALUE_STRING: i32 = 3;

impl EchoValue {
    pub const fn null() -> Self {
        Self {
            kind: ECHO_VALUE_NULL,
            payload: 0,
        }
    }

    pub const fn is_null(self) -> bool {
        self.kind == ECHO_VALUE_NULL
    }

    pub fn string(value: *mut EchoString) -> Self {
        Self {
            kind: ECHO_VALUE_STRING,
            payload: value as u64,
        }
    }

    pub const fn is_string(self) -> bool {
        self.kind == ECHO_VALUE_STRING
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
pub unsafe extern "C" fn echo_value_string(ptr: *const u8, len: usize) -> EchoValue {
    if ptr.is_null() && len != 0 {
        return EchoValue {
            kind: -1,
            payload: 0,
        };
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    EchoValue::string(Box::into_raw(Box::new(EchoString { bytes })))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_start() {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start());
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_start_value(callback: EchoValue) -> bool {
    let Ok(callback) = echo_normalize_callable(callback) else {
        return false;
    };

    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start_with_callback(callback));
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_clean() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_clean())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_flush() -> bool {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let ok = runtime.borrow_mut().ob_flush(&mut stdout);
        write_stdout(&stdout);
        ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_end_flush() -> bool {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let ok = runtime.borrow_mut().ob_end_flush(&mut stdout);
        write_stdout(&stdout);
        ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_end_clean() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_end_clean())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_get_clean() -> *mut EchoString {
    OUTPUT.with(|runtime| match runtime.borrow_mut().ob_get_clean() {
        Some(value) => Box::into_raw(Box::new(value)),
        None => std::ptr::null_mut(),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_get_contents() -> *mut EchoString {
    OUTPUT.with(|runtime| match runtime.borrow().ob_get_contents() {
        Some(value) => Box::into_raw(Box::new(value)),
        None => std::ptr::null_mut(),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_get_flush() -> *mut EchoString {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let value = runtime.borrow_mut().ob_get_flush(&mut stdout);
        write_stdout(&stdout);
        match value {
            Some(value) => Box::into_raw(Box::new(value)),
            None => std::ptr::null_mut(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_get_level() -> i64 {
    // PHP `ob_get_level()` returns zero when inactive; the first active buffer is level 1.
    // Source: https://www.php.net/manual/en/function.ob-get-level.php
    OUTPUT.with(|runtime| runtime.borrow().level() as i64)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_ob_get_length() -> i64 {
    OUTPUT.with(|runtime| {
        runtime
            .borrow()
            .ob_get_length()
            .map_or(-1, |len| len as i64)
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

    #[test]
    fn writes_to_stdout_without_buffer() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        runtime.write(b"hello", &mut stdout);

        assert_eq!(stdout, b"hello");
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

    #[test]
    fn runtime_function_declarations_contain_symbols() {
        for function in RuntimeFn::ALL {
            assert!(function.llvm_decl().contains(function.symbol()));
        }
    }
}
