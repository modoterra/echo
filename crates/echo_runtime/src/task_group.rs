use crate::task::{self, EchoTask};
use crate::{EchoList, EchoValue, sched};
use std::io;

#[derive(Debug, Default)]
pub struct EchoTaskGroup {
    pub(crate) tasks: Vec<*mut EchoTask>,
}

impl EchoTaskGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, task: *mut EchoTask) {
        self.tasks.push(task);
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_group_new() -> EchoValue {
    EchoValue::task_group(Box::into_raw(Box::new(EchoTaskGroup::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_group_add(group_value: EchoValue, task_value: EchoValue) -> EchoValue {
    let Some(group) = group_value.as_task_group_mut() else {
        return EchoValue::error();
    };
    let Some(task) = task_value.as_task_mut() else {
        return EchoValue::error();
    };

    group.add(task as *mut EchoTask);
    group_value
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_task_group_run_and_join(group_value: EchoValue) -> EchoValue {
    let Some(group) = group_value.as_task_group_mut() else {
        return EchoValue::error();
    };

    let schedule_result = sched::with_thread_event_loop(|event_loop| {
        for task in &group.tasks {
            let Some(task) = (unsafe { task.as_ref() }) else {
                return Err(io::Error::other("invalid Echo task in group"));
            };
            event_loop
                .schedule_task(task)
                .map_err(|_| io::Error::other("failed to schedule Echo task group task"))?;
        }
        Ok(())
    });
    if schedule_result.is_err() {
        return EchoValue::error();
    }

    let mut results = EchoList::new();
    for task in &group.tasks {
        let Some(task) = (unsafe { task.as_ref() }) else {
            return EchoValue::error();
        };
        let result = match task.result() {
            Ok(value) => value,
            Err(task::TaskResultError::Failed) => return EchoValue::error(),
            Err(task::TaskResultError::NotFinished) => {
                match sched::with_thread_event_loop(|event_loop| {
                    event_loop
                        .join_task(task)
                        .map_err(|_| io::Error::other("failed to join Echo task group task"))
                }) {
                    Ok(value) => value,
                    Err(_) => return EchoValue::error(),
                }
            }
        };
        results.values.push(result);
    }

    EchoValue::list(Box::into_raw(Box::new(results)))
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn callback() -> EchoValue {
        EchoValue::int(42)
    }

    #[test]
    fn task_group_adds_task_and_returns_group() {
        let group = echo_task_group_new();
        let task = task::echo_task_defer(Some(callback));
        let updated = echo_task_group_add(group, task);

        assert_eq!(updated, group);
        assert_eq!(group.as_task_group_mut().expect("task group").len(), 1);
    }

    #[test]
    fn task_group_run_and_join_returns_list() {
        let group = echo_task_group_new();
        let task = task::echo_task_defer(Some(callback));
        let group = echo_task_group_add(group, task);
        let results = echo_task_group_run_and_join(group);

        assert!(results.is_list());
    }

    #[test]
    fn task_group_add_rejects_invalid_values() {
        assert_eq!(
            echo_task_group_add(EchoValue::int(1), EchoValue::int(2)),
            EchoValue::error()
        );
    }
}
