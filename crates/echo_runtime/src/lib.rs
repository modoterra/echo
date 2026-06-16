use std::cell::RefCell;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFn {
    EchoWrite,
    ObStart,
    ObClean,
    ObFlush,
    ObEndFlush,
    ObEndClean,
    Shutdown,
}

impl RuntimeFn {
    pub const ALL: &'static [Self] = &[
        Self::EchoWrite,
        Self::ObStart,
        Self::ObClean,
        Self::ObFlush,
        Self::ObEndFlush,
        Self::ObEndClean,
        Self::Shutdown,
    ];

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::EchoWrite => "echo_write",
            Self::ObStart => "echo_ob_start",
            Self::ObClean => "echo_ob_clean",
            Self::ObFlush => "echo_ob_flush",
            Self::ObEndFlush => "echo_ob_end_flush",
            Self::ObEndClean => "echo_ob_end_clean",
            Self::Shutdown => "echo_shutdown",
        }
    }

    pub const fn llvm_decl(self) -> &'static str {
        match self {
            Self::EchoWrite => "declare void @echo_write(ptr, i64)",
            Self::ObStart => "declare void @echo_ob_start()",
            Self::ObClean => "declare i1 @echo_ob_clean()",
            Self::ObFlush => "declare i1 @echo_ob_flush()",
            Self::ObEndFlush => "declare i1 @echo_ob_end_flush()",
            Self::ObEndClean => "declare i1 @echo_ob_end_clean()",
            Self::Shutdown => "declare void @echo_shutdown()",
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

    pub fn ob_clean(&mut self) -> bool {
        // PHP `ob_clean()` discards active buffer contents without turning the buffer off.
        // Source: https://www.php.net/manual/en/function.ob-clean.php
        let Some(buffer) = self.stack.last_mut() else {
            return false;
        };

        buffer.clear();
        true
    }

    pub fn ob_flush(&mut self, stdout: &mut Vec<u8>) -> bool {
        if self.stack.is_empty() {
            return false;
        };

        // PHP flushes only the active buffer; nested buffers flush to their parent.
        // Sources: function.ob-flush.php and outcontrol.nesting-output-buffers.php
        let top = self.stack.len() - 1;
        let bytes = std::mem::take(&mut self.stack[top]);

        match top
            .checked_sub(1)
            .and_then(|parent| self.stack.get_mut(parent))
        {
            Some(parent) => parent.extend_from_slice(&bytes),
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

        self.write(&buffer, stdout);
        true
    }

    pub fn ob_end_clean(&mut self) -> bool {
        // PHP `ob_end_clean()` discards contents and turns off the active buffer.
        // Source: https://www.php.net/manual/en/function.ob-end-clean.php
        self.stack.pop().is_some()
    }

    pub fn shutdown(&mut self, stdout: &mut Vec<u8>) {
        // PHP shutdown flushes and turns off still-open buffers in reverse start order.
        // Source: https://www.php.net/manual/en/outcontrol.user-level-output-buffers.php
        while self.ob_end_flush(stdout) {}
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
    fn runtime_function_declarations_contain_symbols() {
        for function in RuntimeFn::ALL {
            assert!(function.llvm_decl().contains(function.symbol()));
        }
    }
}
