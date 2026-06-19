use crate::task::EchoTask;

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
