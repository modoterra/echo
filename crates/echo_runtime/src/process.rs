use crate::EchoValue;
use crate::task::ProcessId;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub struct EchoProcess {
    id: ProcessId,
    command: Vec<u8>,
    child: Child,
}

impl EchoProcess {
    pub fn spawn(id: ProcessId, command: Vec<u8>) -> std::io::Result<Self> {
        let command_string = String::from_utf8_lossy(&command);
        let child = if cfg!(windows) {
            Command::new("cmd")
                .arg("/C")
                .arg(command_string.as_ref())
                .spawn()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command_string.as_ref())
                .spawn()
        }?;

        Ok(Self { id, command, child })
    }

    pub const fn id(&self) -> ProcessId {
        self.id
    }

    pub fn command(&self) -> &[u8] {
        &self.command
    }

    pub fn join(&mut self) -> EchoValue {
        match self.child.wait() {
            Ok(status) => EchoValue::int(status.code().unwrap_or_default() as i64),
            Err(_) => EchoValue::error(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_process_spawn(command: EchoValue) -> EchoValue {
    let Some(command) = command.string_bytes() else {
        return EchoValue::error();
    };
    let id = NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed);

    match EchoProcess::spawn(ProcessId(id), command) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EchoString;

    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes.to_vec()))))
    }

    #[test]
    fn process_spawn_and_join_returns_exit_status() {
        let process = echo_process_spawn(string_value(b"exit 7"));

        assert_eq!(echo_process_join(process), EchoValue::int(7));
    }

    #[test]
    fn process_join_rejects_non_process_values() {
        assert_eq!(echo_process_join(EchoValue::int(7)), EchoValue::error());
    }
}
