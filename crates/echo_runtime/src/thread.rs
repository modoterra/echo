use crate::EchoValue;
use crate::task::{EchoTaskCallback, ThreadId};
use std::thread::JoinHandle;

#[derive(Debug)]
pub struct EchoThread {
    id: ThreadId,
    handle: Option<JoinHandle<EchoValue>>,
}

impl EchoThread {
    pub fn fork(id: ThreadId, callback: EchoTaskCallback) -> Self {
        let handle = std::thread::spawn(move || unsafe { callback() });

        Self {
            id,
            handle: Some(handle),
        }
    }

    pub const fn id(&self) -> ThreadId {
        self.id
    }

    pub fn join(&mut self) -> EchoValue {
        let Some(handle) = self.handle.take() else {
            return EchoValue::error();
        };

        handle.join().unwrap_or_else(|_| EchoValue::error())
    }
}
