use crate::EchoValue;
use crate::task::{EchoTaskCallback, ThreadId};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;

static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(1);

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

#[unsafe(no_mangle)]
pub extern "C" fn echo_thread_fork(callback: Option<EchoTaskCallback>) -> EchoValue {
    let Some(callback) = callback else {
        return EchoValue::error();
    };
    let id = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
    let thread = EchoThread::fork(ThreadId(id), callback);

    EchoValue::thread(Box::into_raw(Box::new(thread)))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_thread_fork_task(task_value: EchoValue) -> EchoValue {
    let Some(task) = task_value.as_task_mut() else {
        return EchoValue::error();
    };
    let Some(callback) = task.callback() else {
        return EchoValue::error();
    };
    let id = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
    let thread = EchoThread::fork(ThreadId(id), callback);

    EchoValue::thread(Box::into_raw(Box::new(thread)))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_thread_join(thread_value: EchoValue) -> EchoValue {
    let Some(thread) = thread_value.as_thread_mut() else {
        return EchoValue::error();
    };

    thread.join()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task;

    unsafe extern "C" fn callback() -> EchoValue {
        EchoValue::int(42)
    }

    #[test]
    fn thread_fork_and_join_returns_callback_value() {
        let thread = echo_thread_fork(Some(callback));

        assert_eq!(echo_thread_join(thread), EchoValue::int(42));
    }

    #[test]
    fn thread_fork_rejects_missing_callback() {
        assert_eq!(echo_thread_fork(None), EchoValue::error());
    }

    #[test]
    fn thread_fork_task_uses_deferred_task_callback() {
        let task = EchoValue::task(Box::into_raw(Box::new(task::EchoTask::deferred(
            task::TaskId(1),
            Some(callback),
        ))));
        let thread = echo_thread_fork_task(task);

        assert_eq!(echo_thread_join(thread), EchoValue::int(42));
    }

    #[test]
    fn thread_join_rejects_non_thread_values() {
        assert_eq!(echo_thread_join(EchoValue::int(42)), EchoValue::error());
    }
}
