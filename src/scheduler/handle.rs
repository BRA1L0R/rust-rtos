use core::ops::{Deref, DerefMut};

use super::{
    arguments::AbiArguments,
    task::{SuspendedTask, Task},
};

/// Type for task handles to be handled in-kernel
/// but are still part of scheduling, hence borrowing
pub struct TaskHandle<'a>(&'a mut Option<Task>);

impl<'a> TaskHandle<'a> {
    pub fn new(task: &'a mut Option<Task>) -> Result<Self, &mut Option<Task>> {
        match task {
            Some(_) => Ok(TaskHandle(task)),
            None => Err(task),
        }
    }

    fn assume(&self) -> &Task {
        // Safety: contract for constructing
        // this type requires Option to be Some
        unsafe { self.0.as_ref().unwrap_unchecked() }
    }

    fn assume_mut(&mut self) -> &mut Task {
        // Safety: contract for constructing
        // this type requires Option to be Some
        unsafe { self.0.as_mut().unwrap_unchecked() }
    }

    pub fn resolve_args<const U: usize>(mut self, args: AbiArguments<U>) {
        let task = self.assume_mut();
        task.apply(args);
    }

    pub fn suspend(self) -> SuspendedTask {
        // Safety: see Self::assume
        let task = unsafe { self.0.take().unwrap_unchecked() };
        SuspendedTask::new(task)
    }
}

impl Deref for TaskHandle<'_> {
    type Target = Task;

    fn deref(&self) -> &Self::Target {
        self.assume()
    }
}

impl DerefMut for TaskHandle<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.assume_mut()
    }
}
