use crate::poll::MioPoller;
use crate::task::{EchoTask, TaskExecuteError, TaskResultError};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::ptr::NonNull;

thread_local! {
    static THREAD_EVENT_LOOP: RefCell<LazyEventLoop> = RefCell::new(LazyEventLoop::new());
}

pub struct EventLoop {
    poller: MioPoller,
    runnable: VecDeque<NonNull<EchoTask>>,
}

impl EventLoop {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            poller: MioPoller::new()?,
            runnable: VecDeque::new(),
        })
    }

    pub fn poller(&self) -> &MioPoller {
        &self.poller
    }

    pub fn poller_mut(&mut self) -> &mut MioPoller {
        &mut self.poller
    }

    pub fn schedule_task(&mut self, task: &mut EchoTask) -> Result<(), TaskExecuteError> {
        task.start().map_err(|_| TaskExecuteError::InvalidState)?;
        self.runnable.push_back(NonNull::from(task));
        Ok(())
    }

    pub fn join_task(&mut self, task: &mut EchoTask) -> Result<crate::EchoValue, TaskResultError> {
        loop {
            match task.result() {
                Ok(value) => return Ok(value),
                Err(TaskResultError::Failed) => return Err(TaskResultError::Failed),
                Err(TaskResultError::NotFinished) => {}
            }

            let Some(runnable) = self.runnable.pop_front() else {
                return Err(TaskResultError::NotFinished);
            };

            self.run_task(runnable)
                .map_err(|_| TaskResultError::Failed)?;
        }
    }

    pub fn has_runnable_tasks(&self) -> bool {
        !self.runnable.is_empty()
    }

    fn run_task(&mut self, mut task: NonNull<EchoTask>) -> Result<(), TaskExecuteError> {
        let task = unsafe { task.as_mut() };
        task.run().map_err(|_| TaskExecuteError::InvalidState)?;

        let Some(callback) = task.callback() else {
            task.fail(crate::EchoError::InvalidCallable);
            return Err(TaskExecuteError::MissingCallback);
        };

        let value = unsafe { callback() };
        task.finish(value);
        Ok(())
    }
}

#[derive(Default)]
pub struct LazyEventLoop {
    event_loop: Option<EventLoop>,
}

impl LazyEventLoop {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_allocated(&self) -> bool {
        self.event_loop.is_some()
    }

    pub fn get_or_init(&mut self) -> io::Result<&mut EventLoop> {
        if self.event_loop.is_none() {
            self.event_loop = Some(EventLoop::new()?);
        }

        Ok(self
            .event_loop
            .as_mut()
            .expect("event loop was initialized"))
    }
}

pub fn with_thread_event_loop<T>(f: impl FnOnce(&mut EventLoop) -> io::Result<T>) -> io::Result<T> {
    THREAD_EVENT_LOOP.with(|event_loop| {
        let mut event_loop = event_loop.borrow_mut();
        f(event_loop.get_or_init()?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EchoValue;
    use crate::task::TaskId;

    #[test]
    fn event_loop_is_allocated_on_demand() {
        let mut event_loop = LazyEventLoop::new();

        assert!(!event_loop.is_allocated());

        event_loop.get_or_init().expect("event loop");

        assert!(event_loop.is_allocated());
    }

    #[test]
    fn thread_event_loop_is_reused_within_thread() {
        let first = with_thread_event_loop(|event_loop| Ok(event_loop as *mut EventLoop as usize))
            .expect("first event loop");
        let second = with_thread_event_loop(|event_loop| Ok(event_loop as *mut EventLoop as usize))
            .expect("second event loop");

        assert_eq!(first, second);
    }

    #[test]
    fn event_loop_schedules_and_joins_task() {
        unsafe extern "C" fn callback() -> EchoValue {
            EchoValue::int(42)
        }

        let mut event_loop = EventLoop::new().expect("event loop");
        let mut task = EchoTask::deferred(TaskId(1), Some(callback));

        event_loop.schedule_task(&mut task).expect("schedule task");

        assert!(event_loop.has_runnable_tasks());
        assert_eq!(task.result(), Err(TaskResultError::NotFinished));

        let result = event_loop.join_task(&mut task).expect("join task");

        assert_eq!(result, EchoValue::int(42));
        assert!(!event_loop.has_runnable_tasks());
    }
}
