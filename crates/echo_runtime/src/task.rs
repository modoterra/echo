use crate::{EchoError, EchoValue};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoTask {
    id: TaskId,
    state: TaskState,
}

impl EchoTask {
    pub fn deferred(id: TaskId) -> Self {
        Self {
            id,
            state: TaskState::Deferred,
        }
    }

    pub const fn id(&self) -> TaskId {
        self.id
    }

    pub const fn state(&self) -> &TaskState {
        &self.state
    }

    pub fn start(&mut self) -> Result<(), TaskStartError> {
        match self.state {
            TaskState::Deferred => {
                self.state = TaskState::Runnable;
                Ok(())
            }
            _ => Err(TaskStartError::NotDeferred),
        }
    }

    pub fn run(&mut self) -> Result<(), TaskRunError> {
        match self.state {
            TaskState::Runnable => {
                self.state = TaskState::Running;
                Ok(())
            }
            _ => Err(TaskRunError::NotRunnable),
        }
    }

    pub fn wait(&mut self, reason: WaitReason) -> Result<(), TaskWaitError> {
        match self.state {
            TaskState::Running => {
                self.state = TaskState::Waiting(reason);
                Ok(())
            }
            _ => Err(TaskWaitError::NotRunning),
        }
    }

    pub fn wake(&mut self) -> Result<(), TaskWakeError> {
        match self.state {
            TaskState::Waiting(_) => {
                self.state = TaskState::Runnable;
                Ok(())
            }
            _ => Err(TaskWakeError::NotWaiting),
        }
    }

    pub fn finish(&mut self, value: EchoValue) {
        self.state = TaskState::Finished(value);
    }

    pub fn fail(&mut self, error: EchoError) {
        self.state = TaskState::Failed(error);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_task_starts_runs_waits_wakes_and_finishes() {
        let mut task = EchoTask::deferred(TaskId(7));

        assert_eq!(task.id(), TaskId(7));
        assert_eq!(task.state(), &TaskState::Deferred);

        assert_eq!(task.start(), Ok(()));
        assert_eq!(task.state(), &TaskState::Runnable);

        assert_eq!(task.run(), Ok(()));
        assert_eq!(task.state(), &TaskState::Running);

        let wait = WaitReason::Io {
            token: IoToken(3),
            interest: IoInterest::Readable,
        };
        assert_eq!(task.wait(wait.clone()), Ok(()));
        assert_eq!(task.state(), &TaskState::Waiting(wait));

        assert_eq!(task.wake(), Ok(()));
        assert_eq!(task.state(), &TaskState::Runnable);

        task.finish(EchoValue::int(42));
        assert_eq!(task.state(), &TaskState::Finished(EchoValue::int(42)));
    }

    #[test]
    fn task_rejects_invalid_transitions() {
        let mut task = EchoTask::deferred(TaskId(1));

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
