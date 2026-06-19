use crate::EchoValue;
use crate::task::ProcessId;
use std::process::{Child, Command};

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
