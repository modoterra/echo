use crate::task::ProcessId;
use crate::{EchoValue, echo_runtime_string, write_runtime_output};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(1);

fn shell_command_output(command: &[u8]) -> std::io::Result<std::process::Output> {
    let command_string = String::from_utf8_lossy(command);
    if cfg!(windows) {
        Command::new("cmd")
            .arg("/C")
            .arg(command_string.as_ref())
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command_string.as_ref())
            .output()
    }
}

fn last_output_line(stdout: &[u8]) -> &[u8] {
    let without_trailing_newline = stdout.strip_suffix(b"\n").unwrap_or(stdout);
    without_trailing_newline
        .rsplit(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default()
}

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
pub extern "C" fn echo_php_shell_exec(command: EchoValue) -> EchoValue {
    let Some(command) = command.string_bytes() else {
        return EchoValue::error();
    };

    let Ok(output) = shell_command_output(&command) else {
        return EchoValue::bool(false);
    };
    if output.stdout.is_empty() {
        EchoValue::null()
    } else {
        echo_runtime_string(output.stdout)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_passthru(command: EchoValue) -> EchoValue {
    let Some(command) = command.string_bytes() else {
        return EchoValue::error();
    };

    let Ok(output) = shell_command_output(&command) else {
        return EchoValue::bool(false);
    };
    write_runtime_output(&output.stdout);
    EchoValue::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_system(command: EchoValue) -> EchoValue {
    let Some(command) = command.string_bytes() else {
        return EchoValue::error();
    };

    let Ok(output) = shell_command_output(&command) else {
        return EchoValue::bool(false);
    };
    write_runtime_output(&output.stdout);
    echo_runtime_string(last_output_line(&output.stdout).to_vec())
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

    #[test]
    fn shell_exec_returns_stdout_or_null_for_empty_output() {
        assert_eq!(
            echo_php_shell_exec(string_value(b"printf 'echo-shell'")).string_bytes(),
            Some(b"echo-shell".to_vec())
        );
        assert_eq!(
            echo_php_shell_exec(string_value(b"printf ''")),
            EchoValue::null()
        );
    }

    #[test]
    fn passthru_writes_stdout_and_returns_null() {
        let (result, stdout) = crate::capture_stdout(false, || {
            echo_php_passthru(string_value(b"printf 'raw-pass'"))
        });

        assert_eq!(result, EchoValue::null());
        assert_eq!(stdout, b"raw-pass".to_vec());
    }

    #[test]
    fn system_writes_stdout_and_returns_last_line() {
        let (result, stdout) = crate::capture_stdout(false, || {
            echo_php_system(string_value(b"printf 'first\nlast\n'"))
        });

        assert_eq!(result.string_bytes(), Some(b"last".to_vec()));
        assert_eq!(stdout, b"first\nlast\n".to_vec());
        assert_eq!(
            echo_php_system(string_value(b"printf ''")).string_bytes(),
            Some(Vec::new())
        );
    }
}
