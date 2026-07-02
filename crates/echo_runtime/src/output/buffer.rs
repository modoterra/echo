use std::mem;

use crate::{EchoCallable, EchoString};

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
pub struct OutputBufferStatus {
    pub name: Vec<u8>,
    pub r#type: i64,
    pub flags: i64,
    pub level: i64,
    pub chunk_size: i64,
    pub buffer_size: i64,
    pub buffer_used: i64,
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
        let bytes = mem::take(&mut self.stack[top].bytes);

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
        self.take_active_buffer().map(EchoString::new)
    }

    pub fn ob_get_flush(&mut self, stdout: &mut Vec<u8>) -> Option<EchoString> {
        // PHP `ob_get_flush()` returns the active buffer contents, flushes them, and turns it off.
        // Source: https://www.php.net/manual/en/function.ob-get-flush.php
        let buffer = self.take_active_buffer()?;
        self.write(&buffer, stdout);
        Some(EchoString::new(buffer))
    }

    pub fn ob_get_contents(&self) -> Option<EchoString> {
        // PHP `ob_get_contents()` returns a new string with the active buffer contents.
        // Source: https://www.php.net/manual/en/function.ob-get-contents.php
        self.stack
            .last()
            .map(|buffer| EchoString::new(buffer.bytes.clone()))
    }

    pub fn ob_get_length(&self) -> Option<usize> {
        // PHP `ob_get_length()` returns the active buffer length in bytes.
        // Source: https://www.php.net/manual/en/function.ob-get-length.php
        self.stack.last().map(|buffer| buffer.bytes.len())
    }

    pub fn ob_list_handlers(&self) -> Vec<Vec<u8>> {
        self.stack
            .iter()
            .map(|_buffer| b"default output handler".to_vec())
            .collect()
    }

    pub fn ob_get_status(&self, full_status: bool) -> Vec<OutputBufferStatus> {
        if full_status {
            return self
                .stack
                .iter()
                .enumerate()
                .map(|(level, buffer)| buffer_status(level, buffer))
                .collect();
        }

        self.stack
            .last()
            .map(|buffer| buffer_status(self.stack.len() - 1, buffer))
            .into_iter()
            .collect()
    }

    pub fn shutdown(&mut self, stdout: &mut Vec<u8>) {
        // PHP shutdown flushes and turns off still-open buffers in reverse start order.
        // Source: https://www.php.net/manual/en/outcontrol.user-level-output-buffers.php
        while self.ob_end_flush(stdout) {}
    }

    pub fn level(&self) -> usize {
        self.stack.len()
    }

    #[cfg(test)]
    pub(crate) fn active_callback(&self) -> Option<&EchoCallable> {
        self.stack
            .last()
            .and_then(|buffer| buffer.callback.as_ref())
    }

    fn take_active_buffer(&mut self) -> Option<Vec<u8>> {
        self.stack.pop().map(|buffer| buffer.bytes)
    }
}

fn buffer_status(level: usize, buffer: &OutputBuffer) -> OutputBufferStatus {
    OutputBufferStatus {
        name: b"default output handler".to_vec(),
        r#type: 0,
        flags: 112,
        level: level as i64,
        chunk_size: 0,
        buffer_size: 16_384,
        buffer_used: buffer.bytes.len() as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EchoSymbol;

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
    fn ob_start_with_callback_stores_callback_frame() {
        let mut runtime = OutputRuntime::new();
        let callback = EchoCallable::Function(EchoSymbol::new("filter"));

        runtime.ob_start_with_callback(Some(callback.clone()));

        assert_eq!(runtime.level(), 1);
        assert_eq!(runtime.active_callback(), Some(&callback));
    }

    #[test]
    fn ob_list_handlers_reports_default_handlers_for_active_buffers() {
        let mut runtime = OutputRuntime::new();

        assert!(runtime.ob_list_handlers().is_empty());

        runtime.ob_start();
        runtime.ob_start();

        assert_eq!(
            runtime.ob_list_handlers(),
            vec![
                b"default output handler".to_vec(),
                b"default output handler".to_vec()
            ]
        );
    }

    #[test]
    fn ob_get_status_reports_top_buffer_metadata() {
        let mut runtime = OutputRuntime::new();
        let mut stdout = Vec::new();

        assert!(runtime.ob_get_status(false).is_empty());

        runtime.ob_start();
        runtime.write(b"abc", &mut stdout);

        assert_eq!(
            runtime.ob_get_status(false),
            vec![OutputBufferStatus {
                name: b"default output handler".to_vec(),
                r#type: 0,
                flags: 112,
                level: 0,
                chunk_size: 0,
                buffer_size: 16_384,
                buffer_used: 3,
            }]
        );
    }

    #[test]
    fn output_reset_rewrite_vars_is_noop_success_baseline() {
        assert_eq!(
            crate::echo_php_output_reset_rewrite_vars(),
            crate::EchoValue::bool(true)
        );
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
