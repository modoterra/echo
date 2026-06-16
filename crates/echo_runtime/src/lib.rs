use std::cell::RefCell;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFn {
    EchoWrite,
    ObStart,
    ObFlush,
    ObEndFlush,
    ObEndClean,
}

impl RuntimeFn {
    pub const ALL: &'static [Self] = &[
        Self::EchoWrite,
        Self::ObStart,
        Self::ObFlush,
        Self::ObEndFlush,
        Self::ObEndClean,
    ];

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::EchoWrite => "echo_write",
            Self::ObStart => "echo_ob_start",
            Self::ObFlush => "echo_ob_flush",
            Self::ObEndFlush => "echo_ob_end_flush",
            Self::ObEndClean => "echo_ob_end_clean",
        }
    }

    pub const fn llvm_decl(self) -> &'static str {
        match self {
            Self::EchoWrite => "declare void @echo_write(ptr, i64)",
            Self::ObStart => "declare void @echo_ob_start()",
            Self::ObFlush => "declare i1 @echo_ob_flush()",
            Self::ObEndFlush => "declare i1 @echo_ob_end_flush()",
            Self::ObEndClean => "declare i1 @echo_ob_end_clean()",
        }
    }
}

#[derive(Debug, Default)]
pub struct OutputRuntime {
    stack: Vec<Vec<u8>>,
}

impl OutputRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, bytes: &[u8], stdout: &mut Vec<u8>) {
        match self.stack.last_mut() {
            Some(buffer) => buffer.extend_from_slice(bytes),
            None => stdout.extend_from_slice(bytes),
        }
    }

    pub fn ob_start(&mut self) {
        self.stack.push(Vec::new());
    }

    pub fn ob_flush(&mut self, stdout: &mut Vec<u8>) -> bool {
        let Some(buffer) = self.stack.last_mut() else {
            return false;
        };

        stdout.extend_from_slice(buffer);
        buffer.clear();
        true
    }

    pub fn ob_end_flush(&mut self, stdout: &mut Vec<u8>) -> bool {
        let Some(buffer) = self.stack.pop() else {
            return false;
        };

        self.write(&buffer, stdout);
        true
    }

    pub fn ob_end_clean(&mut self) -> bool {
        self.stack.pop().is_some()
    }

    pub fn level(&self) -> usize {
        self.stack.len()
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
pub extern "C" fn echo_ob_start() {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start());
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

fn write_stdout(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }

    let mut stdout = io::stdout().lock();
    stdout
        .write_all(bytes)
        .expect("failed to write Echo runtime output");
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
    fn runtime_function_declarations_contain_symbols() {
        for function in RuntimeFn::ALL {
            assert!(function.llvm_decl().contains(function.symbol()));
        }
    }
}
