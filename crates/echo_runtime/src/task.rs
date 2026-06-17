use crate::{EchoError, EchoValue};
use std::sync::{Condvar, Mutex};

pub type EchoTaskCallback = unsafe extern "C" fn() -> EchoValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallbackId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IoToken(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoInterest {
    Readable,
    Writable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitReason {
    Io {
        token: IoToken,
        interest: IoInterest,
    },
    TimerMillis(u64),
    Task(TaskId),
    Thread(ThreadId),
    Process(ProcessId),
    Callback(CallbackId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    Deferred,
    Runnable,
    Running,
    Waiting(WaitReason),
    Finished(EchoValue),
    Failed(EchoError),
}

pub struct EchoTask {
    id: TaskId,
    callback: Option<EchoTaskCallback>,
    state: Mutex<TaskState>,
    completed: Condvar,
}

impl EchoTask {
    pub fn deferred(id: TaskId, callback: Option<EchoTaskCallback>) -> Self {
        Self {
            id,
            callback,
            state: Mutex::new(TaskState::Deferred),
            completed: Condvar::new(),
        }
    }

    pub const fn id(&self) -> TaskId {
        self.id
    }

    pub fn state(&self) -> TaskState {
        self.state.lock().expect("task state poisoned").clone()
    }

    pub const fn callback(&self) -> Option<EchoTaskCallback> {
        self.callback
    }

    pub fn run_to_completion(&self) -> Result<EchoValue, TaskExecuteError> {
        self.start().map_err(|_| TaskExecuteError::InvalidState)?;
        self.run().map_err(|_| TaskExecuteError::InvalidState)?;

        let Some(callback) = self.callback else {
            self.fail(EchoError::InvalidCallable);
            return Err(TaskExecuteError::MissingCallback);
        };

        let value = unsafe { callback() };
        self.finish(value);
        Ok(value)
    }

    pub fn result(&self) -> Result<EchoValue, TaskResultError> {
        match &*self.state.lock().expect("task state poisoned") {
            TaskState::Finished(value) => Ok(*value),
            TaskState::Failed(_) => Err(TaskResultError::Failed),
            _ => Err(TaskResultError::NotFinished),
        }
    }

    pub fn wait_for_result(&self) -> Result<EchoValue, TaskResultError> {
        let mut state = self.state.lock().expect("task state poisoned");
        loop {
            match &*state {
                TaskState::Finished(value) => return Ok(*value),
                TaskState::Failed(_) => return Err(TaskResultError::Failed),
                _ => {
                    state = self.completed.wait(state).expect("task state poisoned");
                }
            }
        }
    }

    pub fn start(&self) -> Result<(), TaskStartError> {
        let mut state = self.state.lock().expect("task state poisoned");
        match &*state {
            TaskState::Deferred => {
                *state = TaskState::Runnable;
                Ok(())
            }
            _ => Err(TaskStartError::NotDeferred),
        }
    }

    pub fn run(&self) -> Result<(), TaskRunError> {
        let mut state = self.state.lock().expect("task state poisoned");
        match &*state {
            TaskState::Runnable => {
                *state = TaskState::Running;
                Ok(())
            }
            _ => Err(TaskRunError::NotRunnable),
        }
    }

    pub fn wait(&self, reason: WaitReason) -> Result<(), TaskWaitError> {
        let mut state = self.state.lock().expect("task state poisoned");
        match &*state {
            TaskState::Running => {
                *state = TaskState::Waiting(reason);
                Ok(())
            }
            _ => Err(TaskWaitError::NotRunning),
        }
    }

    pub fn wake(&self) -> Result<(), TaskWakeError> {
        let mut state = self.state.lock().expect("task state poisoned");
        match &*state {
            TaskState::Waiting(_) => {
                *state = TaskState::Runnable;
                Ok(())
            }
            _ => Err(TaskWakeError::NotWaiting),
        }
    }

    pub fn finish(&self, value: EchoValue) {
        *self.state.lock().expect("task state poisoned") = TaskState::Finished(value);
        self.completed.notify_all();
    }

    pub fn fail(&self, error: EchoError) {
        *self.state.lock().expect("task state poisoned") = TaskState::Failed(error);
        self.completed.notify_all();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStartError {
    NotDeferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRunError {
    NotRunnable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWaitError {
    NotRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWakeError {
    NotWaiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskExecuteError {
    InvalidState,
    MissingCallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskResultError {
    NotFinished,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_task_starts_runs_waits_wakes_and_finishes() {
        unsafe extern "C" fn callback() -> EchoValue {
            EchoValue::int(42)
        }

        let task = EchoTask::deferred(TaskId(7), Some(callback));

        assert_eq!(task.id(), TaskId(7));
        assert!(task.callback().is_some());
        assert_eq!(task.state(), TaskState::Deferred);

        assert_eq!(task.start(), Ok(()));
        assert_eq!(task.state(), TaskState::Runnable);

        assert_eq!(task.run(), Ok(()));
        assert_eq!(task.state(), TaskState::Running);

        let wait = WaitReason::Io {
            token: IoToken(3),
            interest: IoInterest::Readable,
        };
        assert_eq!(task.wait(wait.clone()), Ok(()));
        assert_eq!(task.state(), TaskState::Waiting(wait));

        assert_eq!(task.wake(), Ok(()));
        assert_eq!(task.state(), TaskState::Runnable);

        task.finish(EchoValue::int(42));
        assert_eq!(task.state(), TaskState::Finished(EchoValue::int(42)));
    }

    #[test]
    fn task_rejects_invalid_transitions() {
        let task = EchoTask::deferred(TaskId(1), None);

        assert_eq!(task.run(), Err(TaskRunError::NotRunnable));
        assert_eq!(
            task.wait(WaitReason::TimerMillis(1)),
            Err(TaskWaitError::NotRunning)
        );
        assert_eq!(task.wake(), Err(TaskWakeError::NotWaiting));

        assert_eq!(task.start(), Ok(()));
        assert_eq!(task.start(), Err(TaskStartError::NotDeferred));
    }
}
